from enum import Enum, auto
from matplotlib import pyplot as plt
import pandas as pd
import seaborn as sns
from tqdm import tqdm


class Algorithm(Enum):
    TLSH = auto()
    ssdeep = auto()

def augment_similar_hashes():
    file = "../results/similar_hashes.csv"
    df = pd.read_csv(file)

    def extract_base_file(filepath):
        filename = filepath.split("/")[-1]
        if len(filename.split("__")) == 1:
            # No separator means this is the base file
            return filename
        else:
            return filename.split("__")[0]

    def is_base_file(filepath, base_filename):
        filename = filepath.split("/")[-1]
        return filename == base_filename

    df['Base File'] = df['File'].map(extract_base_file)
    df['Is Base'] = df[['File', 'Base File']].apply(lambda row: is_base_file(row[0], row[1]), axis=1)

    # Known similar payloads, rename records with the base file on the left to the one on the right.
    known_similar = {
        'linux-x64-shell_bind_tcp' : 'linux-x64-shell-bind_tcp',
        'linux-x64-shell-bind_tcp': 'linux-x64-meterpreter-bind_tcp',
        'linux-x64-shell-reverse_tcp': 'linux-x64-meterpreter_reverse_tcp',
        'linux-x64-meterpreter-reverse_tcp' : 'linux-x64-meterpreter_reverse_tcp',
        'linux-x64-meterpreter_reverse_http': 'linux-x64-meterpreter_reverse_https',
        'linux-x64-meterpreter_reverse_tcp': 'linux-x64-meterpreter_reverse_https',
    }

    for e1, e2 in known_similar.items():
        df.loc[df['Base File'] == e1, 'Base File'] = e2

    df.to_csv(file, index=False)

def detect_similar(threshold: int, algorithm: Algorithm, comparison_data: pd.DataFrame) -> (float, float):
    """
    :param threshold: If the algorithm is TLSH, the threshold is the value below which files are determined to be
        similar and should be > 0. If the algorithm is ssdeep, the threshold is the value above which files are
        determined to be similar and should be in the range 0-100.
    :return: Tuple of True and False Positive Rates
    """
    if algorithm == Algorithm.TLSH:
        score_column = "TLSH Diff"
        predict_distinct = lambda x: not (x < threshold)
    elif algorithm == Algorithm.ssdeep:
        score_column = "ssdeep Similarity"
        predict_distinct = lambda x: not (x > threshold)
    else:
        raise NotImplementedError()

    # We predict if files are distinct (rather than if they're similar) since the ground truth is also specified using distinctness
    comparison_data["Predicted Distinct"] = comparison_data[score_column].map(predict_distinct)

    actual_positives = comparison_data[comparison_data["Are Distinct"] == False].shape[0]
    true_positives = comparison_data[
        (comparison_data["Are Distinct"] == False)
        & (comparison_data["Predicted Distinct"] == False)
    ].shape[0]

    actual_negatives = comparison_data[comparison_data["Are Distinct"] == True].shape[0]
    false_positives = comparison_data[
        (comparison_data["Are Distinct"] == True)
        & (comparison_data["Predicted Distinct"] == False)
    ].shape[0]

    return (
        float(true_positives) / actual_positives,      # True positive rate
        float(false_positives) / actual_negatives,     # True negative rate
    )

def threshold_experiment(filter_files=None):
    ssdeep_thresholds = [0, 5, 10, 20, 30, 40, 50, 60, 70, 80, 90, 99]
    tlsh_thresholds = [300, 250, 200, 150, 100, 90, 80, 70, 60, 50, 40, 30, 20, 10]

    comparison_file = "../results/comparisons.csv"
    df = pd.read_csv(comparison_file)
    if filter_files is not None:
        df = df[df["File 1"].map(filter_files) == True]

    ssdeep_stats = []
    tlsh_stats = []

    for st in ssdeep_thresholds:
        tpr, fpr = detect_similar(st, Algorithm.ssdeep, df)
        ssdeep_stats.append({
            "threshold": st,
            "TPR": tpr,
            "FPR": fpr,
        })

    for tt in tlsh_thresholds:
        tpr, fpr = detect_similar(tt, Algorithm.TLSH, df)
        tlsh_stats.append({
            "threshold": tt,
            "TPR": tpr,
            "FPR": fpr,
        })

    return tlsh_stats, ssdeep_stats

def roc_curve(filter_files=None, filter_file_2=None):
    ssdeep_threshold_limits = (-1, 100)
    tlsh_threshold_limits = (0, 2500)

    comparison_file = "../results/comparisons.csv"
    df = pd.read_csv(comparison_file)
    if filter_files is not None:
        df = df[df["File 1"].map(filter_files) == True]

    if filter_file_2 is not None:
        df = df[df["File 2"].map(filter_file_2) == True]

    stats = []

    print("Gathering ROC curve data for ssdeep:")
    for threshold in tqdm(range(ssdeep_threshold_limits[0], ssdeep_threshold_limits[1]+1, 1)):
        tpr, fpr = detect_similar(threshold, Algorithm.ssdeep, df)
        stats.append({
            "Algorithm": "ssdeep",
            "threshold": threshold,
            "TPR": tpr,
            "FPR": fpr,
        })

    print("Gathering ROC curve data for TLSH:")
    for threshold in tqdm(range(tlsh_threshold_limits[0], tlsh_threshold_limits[1]+1, 1)):
        tpr, fpr = detect_similar(threshold, Algorithm.TLSH, df)
        stats.append({
            "Algorithm": "TLSH",
            "threshold": threshold,
            "TPR": tpr,
            "FPR": fpr,
        })

    stats_df = pd.DataFrame(stats)
    return sns.relplot(data=stats_df, x="FPR", y="TPR", hue="Algorithm", kind="line")

def permutation_experiment(datafile):
    df = pd.read_csv(datafile)
    df = df.rename(columns={
        "TLSH Diff": "TLSH",
        "ssdeep Similarity": "ssdeep",
    })
    # Reformat dataframe so that each diff or similarity value has its own row for easy plotting
    df = pd.melt(df, id_vars=['Iteration'], value_vars=['TLSH', 'ssdeep'])
    df = df.rename(columns={
        "variable": "Algorithm",
        "value": "Similarity/Difference Score",
    })
    return sns.relplot(data=df, x="Iteration", y="Similarity/Difference Score", hue="Algorithm", kind="line")

def malware_experiment():
    ssdeep_threshold_limits = (-1, 100)
    tlsh_threshold_limits = (0, 2500)

    comparison_file = "../results/comparisons.csv"
    df = pd.read_csv(comparison_file)

    ### Filters
    def is_malware(filepath: str):
        return filepath.startswith("data/malware/")

    def is_base(filepath: str):
        # Permuted files currently end in a digit
        return filepath[-1] not in ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']

    def gen_is_encoding(encoding: str):
        def is_encoding(filepath: str):
            return encoding in filepath
        return is_encoding

    def gen_is_n_iterations(n: int):
        def is_n_iterations(filepath: str):
            return f"__{n}__" in filepath
        return is_n_iterations

    # Only look at malware for this experiment
    df = df[df["File 1"].map(is_malware) == True]

    iter_stats = []
    encoding_stats = []

    for iterations in [1, 5, 10]:
        is_iters_fn = gen_is_n_iterations(iterations)
        filtered_df = df[df["File 1"].map(lambda fp: is_base(fp) or is_iters_fn(fp)) == True]
        filtered_df = filtered_df[filtered_df["File 2"].map(lambda fp: is_base(fp) or is_iters_fn(fp)) == True]
        print("Gathering ROC curve data for ssdeep:")
        for threshold in tqdm(range(ssdeep_threshold_limits[0], ssdeep_threshold_limits[1]+1, 1)):
            tpr, fpr = detect_similar(threshold, Algorithm.ssdeep, filtered_df)
            iter_stats.append({
                "Algorithm": "ssdeep",
                "Iterations": iterations,
                "threshold": threshold,
                "TPR": tpr,
                "FPR": fpr,
            })

        print("Gathering ROC curve data for TLSH:")
        for threshold in tqdm(range(tlsh_threshold_limits[0], tlsh_threshold_limits[1]+1, 1)):
            tpr, fpr = detect_similar(threshold, Algorithm.TLSH, filtered_df)
            iter_stats.append({
                "Algorithm": "TLSH",
                "Iterations": iterations,
                "threshold": threshold,
                "TPR": tpr,
                "FPR": fpr,
            })

    for encoding in ["shikata_ga_nai", "bloxor"]:
        is_encoding_fn = gen_is_encoding(encoding)
        filtered_df = df[df["File 1"].map(lambda fp: is_base(fp) or is_encoding_fn(fp)) == True]
        filtered_df = filtered_df[filtered_df["File 2"].map(lambda fp: is_base(fp) or is_encoding_fn(fp)) == True]
        print("Gathering ROC curve data for ssdeep:")
        for threshold in tqdm(range(ssdeep_threshold_limits[0], ssdeep_threshold_limits[1]+1, 1)):
            tpr, fpr = detect_similar(threshold, Algorithm.ssdeep, filtered_df)
            encoding_stats.append({
                "Algorithm": "ssdeep",
                "Encoding": encoding,
                "threshold": threshold,
                "TPR": tpr,
                "FPR": fpr,
            })

        print("Gathering ROC curve data for TLSH:")
        for threshold in tqdm(range(tlsh_threshold_limits[0], tlsh_threshold_limits[1]+1, 1)):
            tpr, fpr = detect_similar(threshold, Algorithm.TLSH, filtered_df)
            encoding_stats.append({
                "Algorithm": "TLSH",
                "Encoding": encoding,
                "threshold": threshold,
                "TPR": tpr,
                "FPR": fpr,
            })

    iter_stats_df = pd.DataFrame(iter_stats)
    encoding_stats_df = pd.DataFrame(encoding_stats)

    iter_plot = sns.relplot(data=iter_stats_df, x="FPR", y="TPR", hue="Algorithm", kind="line", style="Iterations")
    encoding_plot = sns.relplot(data=encoding_stats_df, x="FPR", y="TPR", hue="Algorithm", kind="line", style="Encoding")

    return iter_plot, encoding_plot

if __name__ == "__main__":
    from pprint import pprint

    augment_similar_hashes()

    plot = permutation_experiment("../results/pg1342_500.txt")
    plot = plot.set_titles("Small permutations of 500 lines of Pride and Prejudice")
    plot = plot.tight_layout()
    plot.savefig("../results/pg1342_500.png")

    plot = permutation_experiment("../results/pg1342.txt")
    plot = plot.set_titles("Large permutations of Pride and Prejudice")
    plot = plot.tight_layout()
    plot.savefig("../results/pg1342.png")



    ### Determines and prints out True Positive Rates and False Positive Rates using select thresholds
    def is_malware(filepath: str):
        return filepath.startswith("data/malware/")

    malware_tlsh_stats, malware_ssdeep_stats = threshold_experiment(is_malware)

    print("TLSH Stats (on malware):")
    pprint(malware_tlsh_stats)
    print()

    print("ssdeep Stats (on malware):")
    pprint(malware_ssdeep_stats)
    print()

    benign_tlsh_stats, benign_ssdeep_stats = threshold_experiment(lambda s: not (is_malware(s)))

    print("TLSH Stats (on benign bins):")
    pprint(benign_tlsh_stats)
    print()

    print("ssdeep Stats (on benign bins):")
    pprint(benign_ssdeep_stats)
    print()



    ### ROC curves for the malware payloads and benign bins
    plot = roc_curve()
    plot.savefig("../results/full_roc.png")

    plot = roc_curve(is_malware)
    plot.savefig("../results/malware_roc.png")

    plot = roc_curve(lambda s: not (is_malware(s)))
    plot.savefig("../results/benign_roc.png")


    ### ROC curves for the malware payloads that differentiate between encodings and encoding iterations
    iter_plot, encoding_plot = malware_experiment()
    iter_plot.savefig(f"../results/malware_iterations.png")
    encoding_plot.savefig(f"../results/malware_encodings.png")
