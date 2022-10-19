import subprocess
from os import path
from typing import List
import logging
from concurrent.futures import ProcessPoolExecutor

# Enable INFO logging
logging.basicConfig(level=logging.INFO)

def msfvenom(args) -> subprocess.CompletedProcess:
    cmd = ["msfvenom", *args]
    logging.info(f"Running: {' '.join(cmd)}")
    return subprocess.run(cmd, capture_output=True, text=True, check=True)

def get_payloads() -> List[str]:
    all_payloads = msfvenom(["-l", "payloads"]).stdout

    payloads = []
    header_found = False

    for line in all_payloads.splitlines():
        # discard header information
        if not header_found:
            if line.strip().startswith("----"):
                header_found = True
            continue

        # The payload name is listed first, followed by an english description of the payload
        fields = line.strip().split()
        if len(fields) > 1:
            payload = line.strip().split()[0]
            payloads.append(payload)

    return payloads

def get_linux_intel_payloads() -> List[str]:
    linux_intel_prefixes = ["linux/x64/", "linux/x86/"]
    payloads = get_payloads()
    payloads = filter(
        lambda p: any([p.startswith(prefix) for prefix in linux_intel_prefixes]),
        payloads
    )
    return list(payloads)

def generate_payload_variations(payload: str, repeats=5, output_dir="data/malware", encoders=None, iterations=None, arch="x64", platform="linux", format="elf"):
    """
    :param payload: Name of the metasploit payload to generate and save.
    :param repeats: Number of times to repeat the encoding experiment. Polymorphic and metamorphic encodings will give a
        different result when run again.
    :param output_dir:
    :param encoders:
    :param iterations: Number of iterations to repeat the encoding within a single experiment.
    :param arch:
    :param platform:
    :param format:
    :return:
    """
    # Defaults
    if encoders is None:
        encoders = ["x86/shikata_ga_nai", "x86/bloxor"]
    if iterations is None:
        iterations = [1, 5, 10]

    common_args = ["-p", payload, "-a", arch, "--platform", platform, "-f", format]

    # First generate the base payload without any encoding or encryption:
    filename = f"{payload}"
    # Replace slashes with dashes so the full payload name is the filename
    filename = filename.replace("/", "-")
    outpath = path.join(output_dir, filename)
    msfvenom([*common_args, "-o", outpath])

    for enc in encoders:
        for iters in iterations:
            for repeat in range(repeats):
                filename = f"{payload}__{enc}__{iters}__{repeat}"
                # Replace slashes with dashes so the full payload name is the filename
                filename = filename.replace("/", "-")
                outpath = path.join(output_dir, filename)
                msfvenom([*common_args, "-e", enc, "-i", str(iters), "-o", outpath])

def generate_linux_intel_payloads(workers=4):
    payloads = get_linux_intel_payloads()

    with ProcessPoolExecutor(max_workers=workers) as pool:
        pool.map(generate_payload_variations, payloads)

