from alphagenome.data import genome
from alphagenome.models import dna_client, variant_scorers

API_KEY = "AIzaSyBPlbw7KIpienFMpPsDLpt7SAxeqjBsVkA"

# Use the SAME method as biomcp CLI
model = dna_client.create(API_KEY)

# Same variant: chr3:197081044:TACTC>T
chromosome = "chr3"
position = 197081044
reference = "TACTC"
alternate = "T"
interval_size = 131072

# Same calculations as biomcp CLI
half_size = interval_size // 2
interval_start = max(0, position - half_size - 1)  # Convert to 0-based
interval_end = interval_start + interval_size

print(f"Testing variant: {chromosome}:{position} {reference}>{alternate}")
print(f"Interval: {interval_start}-{interval_end} (size: {interval_size})")

# Create interval and variant objects (same as biomcp CLI)
interval = genome.Interval(
    chromosome=chromosome, start=interval_start, end=interval_end
)

variant = genome.Variant(
    chromosome=chromosome,
    position=position,
    reference_bases=reference,
    alternate_bases=alternate,
)

# Get recommended scorers for human (same as biomcp CLI)
scorers = variant_scorers.get_recommended_scorers(organism="human")
print(f"Number of scorers: {len(scorers)}")

# Make prediction using the SAME method as biomcp CLI
scores = model.score_variant(
    interval=interval, variant=variant, variant_scorers=scorers
)

print(f"Number of scores returned: {len(scores)}")

# Convert to DataFrame (same as biomcp CLI)
scores_df = variant_scorers.tidy_scores(scores)
print(f"Scores DataFrame shape: {scores_df.shape}")

if not scores_df.empty:
    print(f"Columns: {list(scores_df.columns)}")
    print(f"Output types: {scores_df['output_type'].unique()}")

    # Gene expression effects (same logic as biomcp CLI)
    expr_scores = scores_df[
        scores_df["output_type"].str.contains("RNA_SEQ", na=False)
    ]
    print(f"\nRNA_SEQ scores found: {len(expr_scores)}")

    if not expr_scores.empty:
        print("Top 10 RNA_SEQ effects:")
        top_expr = expr_scores.nlargest(10, "raw_score")
        for _, row in top_expr.iterrows():
            gene = row.get("gene_name", "Unknown")
            score = row["raw_score"]
            direction = "↓" if score < 0 else "↑"
            print(f"  {gene}: {score:+.2f} log₂ fold change {direction}")
    else:
        print("No RNA_SEQ scores found")

    # Check all significant effects
    significant = scores_df[scores_df["raw_score"].abs() > 0.5]
    print(f"\nSignificant effects (|score| > 0.5): {len(significant)}")

    if not significant.empty:
        print("Top 10 most significant effects:")
        # Sort by absolute value of raw_score
        significant_sorted = significant.iloc[
            (-significant["raw_score"].abs()).argsort()
        ]
        top_sig = significant_sorted.head(10)
        for _, row in top_sig.iterrows():
            output_type = row["output_type"]
            track = row.get("track_name", "unknown")
            gene = row.get("gene_name", "")
            score = row["raw_score"]
            direction = "↓" if score < 0 else "↑"
            print(f"  {output_type}: {gene} {track} {score:+.2f} {direction}")
else:
    print("No scores returned!")
