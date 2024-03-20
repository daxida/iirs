import os

import plotly.express as px  # To silence deprecation warning
import numpy as np
import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots


def main():
    file_path = os.path.join("bench", "results.csv")

    results = pd.read_csv(file_path)
    sizes = results["size_fasta"].unique()
    for size in sizes:
        create_heatmap_at_size(results, size)


def create_heatmap_at_size(results, size):
    results = results[results["size_fasta"] == size]

    fig = make_subplots(rows=3, cols=2, subplot_titles=("Rust", "CPP"))

    title_text = f"Heatmaps of Rust vs CPP performance at size = {size}"
    fig.update_layout(title_text=title_text)

    zmin1, zmax1 = add_heatmaps(fig, results, "min_len", "max_gap", row=1)
    zmin2, zmax2 = add_heatmaps(fig, results, "min_len", "mismatches", row=2)
    zmin3, zmax3 = add_heatmaps(fig, results, "max_gap", "mismatches", row=3)

    zmin = min(zmin1, zmin2, zmin3)
    zmax = min(zmax1, zmax2, zmax3)

    fig.update_traces(zmin=zmin, zmax=zmax)
    # Requires: pip install -U kaleido
    fig.write_image(f"bench/heatmaps/heatmaps_{size}.png")
    fig.show()


def add_heatmaps(fig, results, var1, var2, row):
    df_rs = add_heatmap(fig, "rust_timing", results, var1, var2, row, col=1)
    df_cpp = add_heatmap(fig, "cpp_timing", results, var1, var2, row, col=2)

    zmin = min(np.min(df_cpp), np.min(df_rs))
    zmax = max(np.max(df_cpp), np.max(df_rs))

    return (zmin, zmax)


def add_heatmap(fig, value, results, var1, var2, row, col):
    df = results.pivot_table(index=var1, columns=var2, values=value, aggfunc="mean")

    heatmap = go.Heatmap(
        x=df.index,
        y=df.columns,
        z=df.values,
        name="heatmap",
    )
    fig.add_trace(heatmap, row=row, col=col)

    fig.update_xaxes(title_text=var1, row=row, col=col)
    fig.update_yaxes(title_text=var2, row=row, col=col)

    return df


if __name__ == "__main__":
    main()
