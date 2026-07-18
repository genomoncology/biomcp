//! Tensor parsing and result-shaping tests. Pure: feed tensor structs and bytes
//! into local helpers. No network.

use super::*;

#[test]
fn dtype_and_half_precision_helpers_work() {
    assert_eq!(dtype_size(2), Some(4));
    assert_eq!(dtype_size(999), None);

    let one = f16_to_f32(0x3C00); // 1.0 in IEEE-754 half
    assert!((one - 1.0).abs() < 1e-6);
}

#[test]
fn summarize_tensor_maps_best_gene_and_value() {
    let tensor = alphagenome_proto::Tensor {
        shape: vec![2, 2],
        data_type: alphagenome_proto::DataType::Float32 as i32,
        payload: Some(alphagenome_proto::tensor::Payload::Array(float32_chunk(&[
            0.1, -0.4, 0.7, -0.2,
        ]))),
    };
    let chunks = match tensor.payload.as_ref().unwrap() {
        alphagenome_proto::tensor::Payload::Array(chunk) => vec![chunk.clone()],
        _ => unreachable!("test tensor uses an inline chunk"),
    };
    let variant_data = alphagenome_proto::VariantData {
        values: Some(tensor.clone()),
        metadata: Some(alphagenome_proto::VariantMetadata {
            variant: None,
            track_metadata: Vec::new(),
            gene_metadata: vec![
                alphagenome_proto::GeneScorerMetadata {
                    gene_id: "ENSG000001".to_string(),
                    name: Some("GENE1".to_string()),
                    strand: None,
                    r#type: None,
                    junction_start: None,
                    junction_end: None,
                },
                alphagenome_proto::GeneScorerMetadata {
                    gene_id: "ENSG000002".to_string(),
                    name: Some("GENE2".to_string()),
                    strand: None,
                    r#type: None,
                    junction_start: None,
                    junction_end: None,
                },
            ],
        }),
    };

    let summary = summarize_tensor(&tensor, &chunks, Some(&variant_data)).unwrap();

    assert!((summary.best_value.unwrap() - 0.7).abs() < 1e-6);
    assert_eq!(summary.best_gene.as_deref(), Some("GENE2"));
}

#[test]
fn decompress_tensor_bytes_rejects_oversized_chunk() {
    let chunks = vec![alphagenome_proto::TensorChunk {
        data: vec![0_u8; MAX_TENSOR_CHUNK_DECOMPRESSED_BYTES + 1],
        compression_type: 0,
    }];
    let err = decompress_tensor_bytes(&chunks).unwrap_err();

    assert!(format!("{err:?}").contains("Tensor chunk exceeded"));
}
