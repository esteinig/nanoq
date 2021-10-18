#!/usr/bin/env python

import pandas
import seaborn
from pathlib import Path
from matplotlib import pyplot as plt

def main():

    data = read_data()
    summary = get_data_summary(data=data)

    with pandas.option_context('display.max_rows', None):
        print(data)
        print(summary)

    summary.to_csv('benchmark_summary.tsv', sep='\t', index=False)

    plot_data(data=data)

def get_data_summary(data: pandas.DataFrame) -> pandas.DataFrame:

    data['mem'] = data['mem'] / 1000  # MB
    data = data.sort_values(['task', 'file', 'tool'])

    # Summary per category
    summary = []
    for task, _data in data.groupby("task"):
        for file, __data in _data.groupby("file"):
            for tool, ___data in __data.groupby("tool"):
                mean_seconds = ___data['time'].mean()
                std_seconds = ___data['time'].std()
                mean_mem = ___data['mem'].mean()
                std_mem = ___data['mem'].std()
                summary.append([task, file, tool, mean_seconds, std_seconds, mean_mem, std_mem])

    return pandas.DataFrame(summary, columns=['task', 'file', 'tool', 'time', 'time_sd', 'mem', 'mem_sd'])


def read_data() -> pandas.DataFrame:

    data = []
    for f in Path('benchmarks').glob("*"):
        df = pandas.read_csv(f, header=None, sep=" ", names=['time', 'mem'])

        tool, task, mode = f.name.split("_")

        if tool == "nanostat8":
            tool = "nanostat-t8"
        elif tool == "nanoqf":
            tool = "nanoq-fast"
        elif tool == "rbt":
            tool = "rust-bio-tools"
        elif tool == "seqtk":
            tool = "seqtk-fqchk"
    

        df1 = df.iloc[1:11, :].copy()  # cold start
        df1['tool'] = [tool for _ in df1.iterrows()]
        df1['task'] = [task for _ in df1.iterrows()]
        df1['file'] = [f'zymo_{mode}' for _ in df1.iterrows()]

        data.append(df1)

        if f.name.endswith('fq'):
            df2 = df.iloc[12:, :].copy()
            df2['tool'] = [tool for _ in df2.iterrows()]
            df2['task'] = [task for _ in df2.iterrows()]
            df2['file'] = ['zymo_full' for _ in df2.iterrows()]

            data.append(df2)
        

    return pandas.concat(data)

def plot_data(data: pandas.DataFrame) -> None:

    # Zymo

    fig1, axes1 = plt.subplots(
        nrows=2, ncols=2, figsize=(
            2 * 7, 2 * 4.5
        )
    )

    data1 = data[data['file'].isin(("zymo_fq", "zymo_gz"))]

    filter_data = data1[data1['task'] == 'filt']
    stats_data = data1[data1['task'] == 'stat']

    seaborn.barplot(
        y='time', x='file', hue='tool', data=stats_data, order=['zymo_fq', 'zymo_gz'],
        ax=axes1[0][0], palette=["#de8f05", "#ece133", "#029e73", "#0173b2", "#56b4e9", "#cc78bc"], 
        hue_order=["nanostat", "nanostat-t8", "seqtk-fqchk", "nanoq", "nanoq-fast", "rust-bio-tools"]
    )
    
    seaborn.barplot(
        y='time', x='file', hue='tool', data=filter_data, order=['zymo_fq', 'zymo_gz'],
        ax=axes1[1][0], palette=["#de8f05", "#029e73", "#0173b2", "#56b4e9"], 
        hue_order=["nanofilt", "filtlong", "nanoq", "nanoq-fast"]
    )

    axes1[0][0].set_xlabel("")
    axes1[0][0].set_ylabel("seconds\n")
    axes1[1][0].set_xlabel("")
    axes1[1][0].set_ylabel("seconds\n")
    axes1[0][0].title.set_text('Read stats (time)')
    axes1[1][0].title.set_text('Read filter (time)')

    seaborn.barplot(
        y='mem', x='file', hue='tool', data=stats_data, order=['zymo_fq', 'zymo_gz'],
        ax=axes1[0][1], palette=["#de8f05", "#ece133", "#029e73", "#0173b2", "#56b4e9", "#cc78bc"], 
        hue_order=["nanostat", "nanostat-t8", "seqtk-fqchk", "nanoq", "nanoq-fast", "rust-bio-tools"]
    )
    
    seaborn.barplot(
        y='mem', x='file', hue='tool', data=filter_data, order=['zymo_fq', 'zymo_gz'],
        ax=axes1[1][1], palette=["#de8f05", "#029e73", "#0173b2", "#56b4e9"], 
        hue_order=["nanofilt", "filtlong", "nanoq", "nanoq-fast"]
    )


    axes1[0][1].set_xlabel("")
    axes1[0][1].set_ylabel("MB\n")
    axes1[1][1].set_xlabel("")
    axes1[1][1].set_ylabel("MB\n")
    axes1[0][1].title.set_text('Read stats (memory)')
    axes1[1][1].title.set_text('Read filter (memory)')
    
    fig1.suptitle('Zymo subset (100k reads, 400 Mbp)', fontsize=14, fontweight="bold")
    plt.tight_layout()
    fig1.savefig('benchmarks_zymo.png')

    # Zymo FULL

    fig2, axes2 = plt.subplots(
        nrows=2, ncols=2, figsize=(
            2 * 7, 2 * 4.5
        )
    )

    data2 = data[data['file'].isin(("zymo_full",))]

    filter_data = data2[data2['task'] == 'filt']
    stats_data = data2[data2['task'] == 'stat']


    seaborn.barplot(
        y='time', x='tool', data=stats_data, order=["nanostat", "nanostat-t8", "seqtk-fqchk", "nanoq", "nanoq-fast", "rust-bio-tools"],
        ax=axes2[0][0], palette=["#de8f05", "#ece133",  "#029e73", "#0173b2", "#56b4e9", "#cc78bc"],
    )
    
    seaborn.barplot(
        y='time', x='tool', data=filter_data, order=["nanofilt", "filtlong", "nanoq", "nanoq-fast"],
        ax=axes2[1][0], palette=["#de8f05", "#029e73",  "#0173b2", "#56b4e9"],
    )

    axes2[0][0].set_xlabel("")
    axes2[0][0].set_ylabel("seconds\n")
    axes2[1][0].set_xlabel("")
    axes2[1][0].set_ylabel("seconds\n")
    axes2[0][0].title.set_text('Read stats (time)')
    axes2[1][0].title.set_text('Read filter (time)')


    seaborn.barplot(
        y='mem', x='tool', data=stats_data, order=["nanostat", "nanostat-t8", "seqtk-fqchk", "nanoq", "nanoq-fast", "rust-bio-tools"],
        ax=axes2[0][1], palette=["#de8f05", "#ece133", "#029e73", "#0173b2", "#56b4e9", "#cc78bc"],
    )
    
    seaborn.barplot(
        y='mem', x='tool', data=filter_data, order=["nanofilt", "filtlong", "nanoq", "nanoq-fast"],
        ax=axes2[1][1], palette=["#de8f05", "#029e73", "#0173b2", "#56b4e9"], 
    )


    axes2[0][1].set_xlabel("")
    axes2[0][1].set_ylabel("MB\n")
    axes2[1][1].set_xlabel("")
    axes2[1][1].set_ylabel("MB\n")
    axes2[0][1].title.set_text('Read stats (memory)')
    axes2[1][1].title.set_text('Read filter (memory)')

    fig2.suptitle('Zymo full (3.5m reads, 14 Gbp)', fontsize=14, fontweight="bold")
    plt.tight_layout()
    fig2.savefig('benchmarks_zymo_full.png')

main()