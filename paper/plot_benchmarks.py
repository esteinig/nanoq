#!/usr/bin/env python

import pandas
import seaborn
from pathlib import Path
from matplotlib import pyplot as plt

def main():

    data = read_data()
    plot_data(data)

def read_data() -> pandas.DataFrame:

    data = []
    for f in Path('replicate_benchmarks').glob("*"):
        df = pandas.read_csv(f, header=None, sep="\t", names=['time'])

        tool, ftype, mode = f.name.split("_")
        df['tool'] = [tool for _ in df.iterrows()]
        df['ftype'] = [ftype for _ in df.iterrows()]
        df['mode'] = [mode for _ in df.iterrows()]

        data.append(df)

    return pandas.concat(data)

def plot_data(data: pandas.DataFrame) -> None:

    fig, axes = plt.subplots(
        nrows=1, ncols=2, figsize=(
            2 * 7, 1 * 4.5
        )
    )

    filter_data = data[data['mode'] == 'filt']
    stats_data = data[data['mode'] == 'stats']

    seaborn.violinplot(
        y='time', x='ftype', hue='tool', data=filter_data,
        ax=axes[0], palette="Greens"
    )
    seaborn.stripplot(
        y='time', x='ftype', hue='tool', data=filter_data, jitter=True,
        zorder=1, palette="Greys", linewidth=1, ax=axes[0], dodge=True
    )

    seaborn.violinplot(
        y='time', x='ftype', hue='tool', data=stats_data,
        ax=axes[1], palette="Greens"
    )
    seaborn.stripplot(
        y='time', x='ftype', hue='tool', data=stats_data, jitter=True,
        zorder=1, palette="Greys", linewidth=1, ax=axes[1], dodge=True
    )

    plt.tight_layout()
    fig.savefig('benchmarks.png')

main()