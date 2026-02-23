use std::io::Cursor;
use std::time::Duration;

use tokio_stream::StreamExt;

use crate::entities::variant::VariantPrediction;
use crate::error::BioMcpError;

const ALPHAGENOME_API: &str = "alphagenome";
const ALPHAGENOME_BASE_ENV: &str = "BIOMCP_ALPHAGENOME_BASE";
const ALPHAGENOME_API_KEY_ENV: &str = "ALPHAGENOME_API_KEY";
const ALPHAGENOME_ENDPOINT: &str = "https://gdmscience.googleapis.com:443";
const ALPHAGENOME_DOMAIN: &str = "gdmscience.googleapis.com";

const INTERVAL_HALF: i64 = 262_144;
const MAX_TENSOR_DECOMPRESSED_BYTES: usize = 256 * 1024 * 1024;
const MAX_TENSOR_CHUNK_DECOMPRESSED_BYTES: usize = 32 * 1024 * 1024;

pub struct AlphaGenomeClient {
    channel: tonic::transport::Channel,
    api_key: String,
}

impl AlphaGenomeClient {
    pub async fn new() -> Result<Self, BioMcpError> {
        let api_key = std::env::var(ALPHAGENOME_API_KEY_ENV)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                BioMcpError::InvalidArgument(format!(
                    "{ALPHAGENOME_API_KEY_ENV} environment variable is required for `get variant <id> predict`."
                ))
            })?;

        let endpoint_url =
            crate::sources::env_base(ALPHAGENOME_ENDPOINT, ALPHAGENOME_BASE_ENV).into_owned();
        let tls_domain = reqwest::Url::parse(&endpoint_url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .filter(|h| !h.trim().is_empty())
            .unwrap_or_else(|| ALPHAGENOME_DOMAIN.to_string());

        let endpoint = tonic::transport::Endpoint::from_shared(endpoint_url.clone())
            .map_err(|err| BioMcpError::Api {
                api: ALPHAGENOME_API.to_string(),
                message: format!("Invalid endpoint URL {endpoint_url}: {err}"),
            })?
            .tls_config(
                tonic::transport::ClientTlsConfig::new()
                    .with_enabled_roots()
                    .domain_name(tls_domain),
            )
            .map_err(|err| BioMcpError::Api {
                api: ALPHAGENOME_API.to_string(),
                message: format!("TLS config failed: {err}"),
            })?
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(60));

        let channel = endpoint.connect().await.map_err(|err| BioMcpError::Api {
            api: ALPHAGENOME_API.to_string(),
            message: format!("connect failed: {err:?}"),
        })?;

        Ok(Self { channel, api_key })
    }

    pub async fn score_variant(
        &self,
        chromosome: &str,
        position: i64,
        reference: &str,
        alternate: &str,
    ) -> Result<VariantPrediction, BioMcpError> {
        let mut client = alphagenome_proto::dna_model_service_client::DnaModelServiceClient::new(
            self.channel.clone(),
        );

        let interval = make_interval(chromosome, position);
        let variant = alphagenome_proto::Variant {
            chromosome: chromosome.to_string(),
            position,
            reference_bases: reference.to_string(),
            alternate_bases: alternate.to_string(),
        };

        let request = alphagenome_proto::ScoreVariantRequest {
            interval: Some(interval),
            variant: Some(variant),
            organism: alphagenome_proto::Organism::HomoSapiens as i32,
            variant_scorers: recommended_scorers(),
            model_version: String::new(),
        };

        let stream = tokio_stream::iter(vec![request]);
        let mut req = tonic::Request::new(stream);
        req.metadata_mut().insert(
            "x-goog-api-key",
            self.api_key
                .parse()
                .map_err(|_| BioMcpError::InvalidArgument("Invalid ALPHAGENOME_API_KEY".into()))?,
        );

        let mut responses = client
            .score_variant(req)
            .await
            .map_err(|err| BioMcpError::Api {
                api: ALPHAGENOME_API.to_string(),
                message: format!("rpc ScoreVariant failed: {err}"),
            })?
            .into_inner();

        let mut values: Vec<TensorSummary> = Vec::new();

        while let Some(resp) = responses.next().await {
            let resp = resp.map_err(|err| BioMcpError::Api {
                api: ALPHAGENOME_API.to_string(),
                message: format!("rpc stream error: {err}"),
            })?;

            match resp.payload {
                Some(alphagenome_proto::score_variant_response::Payload::Output(out)) => {
                    let tensor = out
                        .variant_data
                        .as_ref()
                        .and_then(|v| v.values.as_ref())
                        .ok_or_else(|| BioMcpError::Api {
                            api: ALPHAGENOME_API.to_string(),
                            message: "Missing ScoreVariantOutput tensor".into(),
                        })?
                        .clone();

                    let chunks = read_tensor_chunks(&mut responses, &tensor).await?;
                    let summary = summarize_tensor(&tensor, &chunks, out.variant_data.as_ref())?;
                    values.push(summary);
                }
                Some(alphagenome_proto::score_variant_response::Payload::TensorChunk(_)) => {
                    return Err(BioMcpError::Api {
                        api: ALPHAGENOME_API.to_string(),
                        message: "Received tensor chunk before output".into(),
                    });
                }
                None => {
                    return Err(BioMcpError::Api {
                        api: ALPHAGENOME_API.to_string(),
                        message: "Empty AlphaGenome response".into(),
                    });
                }
            }
        }

        // Map outputs by scorer order.
        let expression = values.first();
        let splice = values.get(1);
        let chromatin = values.get(2);

        Ok(VariantPrediction {
            expression_lfc: expression.and_then(|s| s.best_value),
            splice_score: splice.and_then(|s| s.best_value),
            chromatin_score: chromatin.and_then(|s| s.best_value),
            top_gene: expression.and_then(|s| s.best_gene.clone()),
        })
    }
}

#[derive(Debug, Clone)]
struct TensorSummary {
    best_value: Option<f64>,
    best_gene: Option<String>,
}

fn recommended_scorers() -> Vec<alphagenome_proto::VariantScorer> {
    vec![
        // Expression: GeneMaskLFCScorer(RNA_SEQ)
        alphagenome_proto::VariantScorer {
            scorer: Some(alphagenome_proto::variant_scorer::Scorer::GeneMask(
                alphagenome_proto::GeneMaskLfcScorer {
                    requested_output: alphagenome_proto::OutputType::RnaSeq as i32,
                },
            )),
        },
        // Splicing: GeneMaskSplicingScorer(SPLICE_SITES)
        alphagenome_proto::VariantScorer {
            scorer: Some(alphagenome_proto::variant_scorer::Scorer::GeneMaskSplicing(
                alphagenome_proto::GeneMaskSplicingScorer {
                    width: None,
                    requested_output: alphagenome_proto::OutputType::SpliceSites as i32,
                },
            )),
        },
        // Chromatin: CenterMaskScorer(DNASE, width=501)
        alphagenome_proto::VariantScorer {
            scorer: Some(alphagenome_proto::variant_scorer::Scorer::CenterMask(
                alphagenome_proto::CenterMaskScorer {
                    width: Some(501),
                    aggregation_type: alphagenome_proto::AggregationType::DiffMean as i32,
                    requested_output: alphagenome_proto::OutputType::Dnase as i32,
                },
            )),
        },
    ]
}

fn make_interval(chr: &str, pos_1based: i64) -> alphagenome_proto::Interval {
    let center_0based = (pos_1based - 1).max(0);
    let start = center_0based.saturating_sub(INTERVAL_HALF).max(0);
    let mut end = center_0based.saturating_add(INTERVAL_HALF);
    // Ensure width is constant (2^19) when clamped at 0.
    if start == 0 {
        end = (2 * INTERVAL_HALF).max(end);
    }

    alphagenome_proto::Interval {
        chromosome: chr.to_string(),
        start,
        end,
        strand: alphagenome_proto::Strand::Unstranded as i32,
    }
}

async fn read_tensor_chunks(
    responses: &mut tonic::Streaming<alphagenome_proto::ScoreVariantResponse>,
    tensor: &alphagenome_proto::Tensor,
) -> Result<Vec<alphagenome_proto::TensorChunk>, BioMcpError> {
    match tensor.payload {
        Some(alphagenome_proto::tensor::Payload::Array(ref arr)) => Ok(vec![arr.clone()]),
        Some(alphagenome_proto::tensor::Payload::ChunkCount(n)) => {
            if n < 0 {
                return Err(BioMcpError::Api {
                    api: ALPHAGENOME_API.to_string(),
                    message: "Invalid tensor chunk count".into(),
                });
            }

            const MAX_CHUNKS: i64 = 1000;
            if n > MAX_CHUNKS {
                return Err(BioMcpError::Api {
                    api: ALPHAGENOME_API.to_string(),
                    message: format!("Tensor chunk count too large: {n} (max {MAX_CHUNKS})"),
                });
            }

            let n_usize = n as usize;
            let mut out: Vec<alphagenome_proto::TensorChunk> = Vec::with_capacity(n_usize);
            for _ in 0..n_usize {
                let resp = responses
                    .next()
                    .await
                    .ok_or_else(|| BioMcpError::Api {
                        api: ALPHAGENOME_API.to_string(),
                        message: "Unexpected end of AlphaGenome stream".into(),
                    })?
                    .map_err(|err| BioMcpError::Api {
                        api: ALPHAGENOME_API.to_string(),
                        message: err.message().to_string(),
                    })?;

                match resp.payload {
                    Some(alphagenome_proto::score_variant_response::Payload::TensorChunk(
                        chunk,
                    )) => out.push(chunk),
                    Some(alphagenome_proto::score_variant_response::Payload::Output(_)) => {
                        return Err(BioMcpError::Api {
                            api: ALPHAGENOME_API.to_string(),
                            message: "Received output while expecting tensor chunks".into(),
                        });
                    }
                    None => {
                        return Err(BioMcpError::Api {
                            api: ALPHAGENOME_API.to_string(),
                            message: "Empty AlphaGenome response while reading chunks".into(),
                        });
                    }
                }
            }
            Ok(out)
        }
        None => Err(BioMcpError::Api {
            api: ALPHAGENOME_API.to_string(),
            message: "Missing tensor payload".into(),
        }),
    }
}

fn summarize_tensor(
    tensor: &alphagenome_proto::Tensor,
    chunks: &[alphagenome_proto::TensorChunk],
    variant_data: Option<&alphagenome_proto::VariantData>,
) -> Result<TensorSummary, BioMcpError> {
    let shape = tensor
        .shape
        .iter()
        .map(|&v| v.max(0) as usize)
        .collect::<Vec<_>>();
    let (layers, obs, vars) = match shape.as_slice() {
        [o, v] => (1usize, *o, *v),
        [l, o, v] => (*l, *o, *v),
        [v] => (1usize, 1usize, *v),
        _ => {
            return Err(BioMcpError::Api {
                api: ALPHAGENOME_API.to_string(),
                message: "Unsupported tensor shape".into(),
            });
        }
    };

    if obs == 0 || vars == 0 || layers == 0 {
        return Ok(TensorSummary {
            best_value: None,
            best_gene: None,
        });
    }

    let bytes = decompress_tensor_bytes(chunks)?;
    let dtype = tensor.data_type;
    let elem_size = dtype_size(dtype).ok_or_else(|| BioMcpError::Api {
        api: ALPHAGENOME_API.to_string(),
        message: format!("Unsupported tensor data_type: {dtype}"),
    })?;

    let first_layer_elems = obs.saturating_mul(vars);
    let required = first_layer_elems.saturating_mul(elem_size);
    if bytes.len() < required {
        return Err(BioMcpError::Api {
            api: ALPHAGENOME_API.to_string(),
            message: "Tensor payload shorter than expected".into(),
        });
    }

    let gene_metadata = variant_data
        .and_then(|v| v.metadata.as_ref())
        .map(|m| &m.gene_metadata)
        .filter(|g| !g.is_empty());

    if let Some(genes) = gene_metadata {
        // Gene-based scorers: identify the top gene by max |score| across tracks.
        let gene_count = obs.min(genes.len());

        let mut best_gene_idx: usize = 0;
        let mut best_abs: f64 = -1.0;
        let mut best_val: f64 = 0.0;

        for gene_idx in 0..gene_count {
            let mut row_best_abs: f64 = -1.0;
            let mut row_best_val: f64 = 0.0;
            for track_idx in 0..vars {
                let flat = gene_idx
                    .checked_mul(vars)
                    .and_then(|v| v.checked_add(track_idx))
                    .ok_or_else(|| BioMcpError::Api {
                        api: ALPHAGENOME_API.to_string(),
                        message: format!(
                            "Tensor index overflow: gene_idx={gene_idx}, vars={vars}, track_idx={track_idx}",
                        ),
                    })?;
                let offset = flat.saturating_mul(elem_size);
                let v = decode_value(dtype, &bytes[offset..offset + elem_size])?;
                let a = v.abs();
                if a > row_best_abs {
                    row_best_abs = a;
                    row_best_val = v;
                }
            }
            if row_best_abs > best_abs {
                best_abs = row_best_abs;
                best_val = row_best_val;
                best_gene_idx = gene_idx;
            }
        }

        let gene = genes.get(best_gene_idx).and_then(|g| {
            g.name
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(|v| v.to_string())
                .or_else(|| {
                    let v = g.gene_id.trim();
                    if v.is_empty() {
                        None
                    } else {
                        Some(v.to_string())
                    }
                })
        });

        Ok(TensorSummary {
            best_value: Some(best_val),
            best_gene: gene,
        })
    } else {
        // Track-only scorers: just return the max |score| across the matrix.
        let mut best_abs: f64 = -1.0;
        let mut best_val: f64 = 0.0;
        for idx in 0..first_layer_elems {
            let offset = idx.saturating_mul(elem_size);
            let v = decode_value(dtype, &bytes[offset..offset + elem_size])?;
            let a = v.abs();
            if a > best_abs {
                best_abs = a;
                best_val = v;
            }
        }
        Ok(TensorSummary {
            best_value: Some(best_val),
            best_gene: None,
        })
    }
}

fn decompress_tensor_bytes(
    chunks: &[alphagenome_proto::TensorChunk],
) -> Result<Vec<u8>, BioMcpError> {
    let mut out: Vec<u8> = Vec::new();
    for chunk in chunks {
        let decompressed = match chunk.compression_type {
            1 => zstd::decode_all(Cursor::new(&chunk.data)).map_err(|err| BioMcpError::Api {
                api: ALPHAGENOME_API.to_string(),
                message: err.to_string(),
            })?,
            _ => chunk.data.clone(),
        };
        if decompressed.len() > MAX_TENSOR_CHUNK_DECOMPRESSED_BYTES {
            return Err(BioMcpError::Api {
                api: ALPHAGENOME_API.to_string(),
                message: format!(
                    "Tensor chunk exceeded {MAX_TENSOR_CHUNK_DECOMPRESSED_BYTES} bytes"
                ),
            });
        }
        let next_len =
            out.len()
                .checked_add(decompressed.len())
                .ok_or_else(|| BioMcpError::Api {
                    api: ALPHAGENOME_API.to_string(),
                    message: "Tensor payload size overflow".into(),
                })?;
        if next_len > MAX_TENSOR_DECOMPRESSED_BYTES {
            return Err(BioMcpError::Api {
                api: ALPHAGENOME_API.to_string(),
                message: format!("Tensor payload exceeded {MAX_TENSOR_DECOMPRESSED_BYTES} bytes"),
            });
        }
        out.extend_from_slice(&decompressed);
    }
    Ok(out)
}

fn dtype_size(data_type: i32) -> Option<usize> {
    match data_type {
        1 => Some(2),  // bfloat16
        2 => Some(4),  // float32
        3 => Some(8),  // float64
        11 => Some(2), // float16
        _ => None,
    }
}

fn decode_value(data_type: i32, bytes: &[u8]) -> Result<f64, BioMcpError> {
    match data_type {
        1 => {
            // bfloat16
            let b0 = bytes.first().copied().unwrap_or(0);
            let b1 = bytes.get(1).copied().unwrap_or(0);
            let bits = u16::from_le_bytes([b0, b1]) as u32;
            Ok(f32::from_bits(bits << 16) as f64)
        }
        2 => {
            let arr: [u8; 4] = bytes
                .get(0..4)
                .ok_or_else(|| BioMcpError::Api {
                    api: ALPHAGENOME_API.to_string(),
                    message: "Invalid float32 tensor bytes".into(),
                })?
                .try_into()
                .expect("slice length checked");
            Ok(f32::from_le_bytes(arr) as f64)
        }
        3 => {
            let arr: [u8; 8] = bytes
                .get(0..8)
                .ok_or_else(|| BioMcpError::Api {
                    api: ALPHAGENOME_API.to_string(),
                    message: "Invalid float64 tensor bytes".into(),
                })?
                .try_into()
                .expect("slice length checked");
            Ok(f64::from_le_bytes(arr))
        }
        11 => {
            let b0 = bytes.first().copied().unwrap_or(0);
            let b1 = bytes.get(1).copied().unwrap_or(0);
            let bits = u16::from_le_bytes([b0, b1]);
            Ok(f16_to_f32(bits) as f64)
        }
        _ => Err(BioMcpError::Api {
            api: ALPHAGENOME_API.to_string(),
            message: format!("Unsupported tensor data_type: {data_type}"),
        }),
    }
}

fn f16_to_f32(bits: u16) -> f32 {
    // Based on IEEE-754 half precision.
    let sign = ((bits >> 15) & 0x1) as u32;
    let exp = ((bits >> 10) & 0x1f) as i32;
    let frac = (bits & 0x03ff) as u32;

    let f32_bits = if exp == 0 {
        if frac == 0 {
            sign << 31
        } else {
            // Subnormal.
            let mut e = -14;
            let mut f = frac;
            while (f & 0x0400) == 0 {
                f <<= 1;
                e -= 1;
            }
            f &= 0x03ff;
            let exp_bits = (e + 127) as u32;
            (sign << 31) | (exp_bits << 23) | (f << 13)
        }
    } else if exp == 0x1f {
        // Inf/NaN.
        (sign << 31) | 0x7f80_0000 | (frac << 13)
    } else {
        // Normal.
        let exp_bits = (exp - 15 + 127) as u32;
        (sign << 31) | (exp_bits << 23) | (frac << 13)
    };

    f32::from_bits(f32_bits)
}

pub(crate) mod alphagenome_proto {
    tonic::include_proto!("google.gdm.gdmscience.alphagenome.v1main");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_interval_clamps_start_and_keeps_expected_width() {
        let interval = make_interval("chr7", 1);
        assert_eq!(interval.chromosome, "chr7");
        assert_eq!(interval.start, 0);
        assert_eq!(interval.end, 2 * INTERVAL_HALF);
    }

    #[test]
    fn recommended_scorers_are_stable() {
        let scorers = recommended_scorers();
        assert_eq!(scorers.len(), 3);
    }

    #[test]
    fn dtype_and_half_precision_helpers_work() {
        assert_eq!(dtype_size(2), Some(4));
        assert_eq!(dtype_size(999), None);
        let one = f16_to_f32(0x3C00); // 1.0 in IEEE-754 half
        assert!((one - 1.0).abs() < 1e-6);
    }

    #[test]
    fn decompress_tensor_bytes_rejects_oversized_chunk() {
        let chunks = vec![alphagenome_proto::TensorChunk {
            data: vec![0_u8; MAX_TENSOR_CHUNK_DECOMPRESSED_BYTES + 1],
            compression_type: 0,
        }];
        let err = decompress_tensor_bytes(&chunks).unwrap_err();
        assert!(format!("{err}").contains("Tensor chunk exceeded"));
    }
}
