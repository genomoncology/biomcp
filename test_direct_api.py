from alphagenome.data import genome
from alphagenome.models import dna_client

API_KEY = "AIzaSyBPlbw7KIpienFMpPsDLpt7SAxeqjBsVkA"
model = dna_client.create(API_KEY)

# Use our test variant: chr3:197081044:TACTC>T
# Create interval around the variant (using default 131,072 bp window like biomcp CLI)
interval = genome.Interval(
    chromosome="chr3",
    start=197081044 - 65536,  # 131,072 bp window centered on variant
    end=197081044 + 65536,
)

variant = genome.Variant(
    chromosome="chr3",
    position=197081044,
    reference_bases="TACTC",
    alternate_bases="T",
)

print(
    f"Testing variant: {variant.chromosome}:{variant.position}:{variant.reference_bases}>{variant.alternate_bases}"
)
print(
    f"Analysis interval: {interval.chromosome}:{interval.start}-{interval.end}"
)

# Try some common ontology terms
common_terms = [
    "UBERON:0000178",  # blood
    "UBERON:0000955",  # brain
    "UBERON:0002048",  # lung
    "UBERON:0000948",  # heart
]

for term in common_terms:
    try:
        print(f"\nTrying ontology term: {term}")
        outputs = model.predict_variant(
            interval=interval,
            variant=variant,
            ontology_terms=[term],
            requested_outputs=[dna_client.OutputType.RNA_SEQ],
        )
        print(f"✅ Success with {term}")
        break
    except Exception as e:
        print(f"❌ Failed with {term}: {e}")
        continue
else:
    print("All ontology terms failed")

print("Direct API Results:")
print(f"Output type: {type(outputs)}")

# Check what attributes the output has
print(f"Output attributes: {dir(outputs)}")

# Access reference and alternate outputs
print(f"Reference output type: {type(outputs.reference)}")
print(f"Alternate output type: {type(outputs.alternate)}")

ref_attrs = dir(outputs.reference)
print(
    f"Reference attributes: {[attr for attr in ref_attrs if not attr.startswith('_')]}"
)

# Look for RNA-seq data in reference/alternate
for name, output in [
    ("reference", outputs.reference),
    ("alternate", outputs.alternate),
]:
    print(f"\n{name} output:")
    if hasattr(output, "rna_seq"):
        rna_data = output.rna_seq
        print(f"  RNA-seq data found: {type(rna_data)}")
        print(
            f"  RNA-seq attributes: {[attr for attr in dir(rna_data) if not attr.startswith('_')]}"
        )

        # Check TrackData structure
        if hasattr(rna_data, "values"):
            print(f"  Values shape: {rna_data.values.shape}")
        if hasattr(rna_data, "track_names"):
            print(f"  Number of tracks: {len(rna_data.track_names)}")
            print(f"  Sample track names: {rna_data.track_names[:5]}")
        if hasattr(rna_data, "metadata"):
            print(f"  Metadata type: {type(rna_data.metadata)}")

# Get gene expression predictions using the proper API
print("\nUsing get() method to access gene expression:")
ref_gene_expr = outputs.reference.get("rna_seq")
alt_gene_expr = outputs.alternate.get("rna_seq")

print(f"Reference gene expression type: {type(ref_gene_expr)}")
print(f"Alternate gene expression type: {type(alt_gene_expr)}")

if ref_gene_expr is not None and alt_gene_expr is not None:
    print(
        f"Reference attributes: {[attr for attr in dir(ref_gene_expr) if not attr.startswith('_')]}"
    )

    # Try to find genes with meaningful data
    if hasattr(ref_gene_expr, "values") and hasattr(alt_gene_expr, "values"):
        import numpy as np

        ref_vals = ref_gene_expr.values
        alt_vals = alt_gene_expr.values

        print(f"Reference shape: {ref_vals.shape}")
        print(f"Alternate shape: {alt_vals.shape}")

        if ref_vals.shape == alt_vals.shape and len(ref_vals.shape) > 1:
            # Calculate differences across genes (assuming genes are in one dimension)
            differences = alt_vals - ref_vals

            # Get gene identifiers
            if hasattr(ref_gene_expr, "track_names"):
                gene_names = ref_gene_expr.track_names
                print(f"Found {len(gene_names)} genes")

                # Sum differences across spatial dimensions to get per-gene effects
                gene_effects = (
                    np.sum(differences, axis=0)
                    if differences.ndim > 1
                    else differences
                )

                # Find top affected genes
                abs_effects = np.abs(gene_effects)
                top_indices = np.argsort(abs_effects)[-10:]

                print("\nTop 10 most affected genes:")
                for idx in reversed(top_indices):
                    if idx < len(gene_names):
                        gene = gene_names[idx]
                        effect = gene_effects[idx]
                        direction = "↑" if effect > 0 else "↓"
                        print(
                            f"  {gene}: {effect:+.2f} cumulative effect {direction}"
                        )
            else:
                print("No gene names found in track_names")
