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

    data = data[data["ftype"] != "crab")]  # exlude bio parser for now, slightly slower than needletail

    for mode, _data in data.groupby("mode"):
        for tool, __data in _data.groupby("tool"):
            for ftype, ___data in _data.groupby("ftype")
            mean_seconds = ___data['time'].mean()
            std_seconds = ___data['time'].std()
            print(f"Mode: {mode} Ftype: {ftype} - Tool: {tool} - Mean: {mean_seconds} - Standard Deviation: {std_seconds}")

    filter_data = data[data['mode'] == 'filt']
    stats_data = data[data['mode'] == 'stat']

    seaborn.barplot(
        y='time', x='ftype', hue='tool', data=filter_data,
        ax=axes[0], palette=["#E69F00", "#56B4E9", "#009E73"]
    )
    
    seaborn.barplot(
        y='time', x='ftype', hue='tool', data=stats_data,
        ax=axes[1], palette=["#E69F00", "#009E73"], hue_order=["nanostat", "nanoq"]
    )

    axes[0].set_xlabel("")
    axes[0].set_ylabel("seconds\n")
    axes[1].set_xlabel("")
    axes[1].set_ylabel("seconds\n")

    axes[0].title.set_text('Read filter')
    axes[1].title.set_text('Read statistics')

    plt.tight_layout()
    fig.savefig('benchmarks.png')

main()