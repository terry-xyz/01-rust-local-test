"""Restore the pinned official test files; no Docker daemon is required."""
import hashlib
import io
import json
from pathlib import Path
import tarfile
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[1]
VENDOR = ROOT / "vendor" / "01-edu-rust"
SOURCE = json.loads((VENDOR / "source.json").read_text())
TOKEN_URL = "https://ghcr.io/token?scope=repository:01-edu/test-rust:pull"
REGISTRY = "https://ghcr.io/v2/01-edu/test-rust"


def fetch(url, headers=None):
    with urlopen(Request(url, headers=headers or {}), timeout=60) as response:
        return response.read()


def verified(data, digest):
    if "sha256:" + hashlib.sha256(data).hexdigest() != digest:
        raise ValueError(f"Digest mismatch: {digest}")
    return data


def main():
    token = json.loads(fetch(TOKEN_URL))["token"]
    headers = {"Authorization": f"Bearer {token}"}
    manifest = json.loads(verified(fetch(
        f"{REGISTRY}/manifests/{SOURCE['digest']}",
        {**headers, "Accept": "application/vnd.docker.distribution.manifest.v2+json"},
    ), SOURCE["digest"]))
    config_digest = manifest["config"]["digest"]
    config = json.loads(verified(fetch(f"{REGISTRY}/blobs/{config_digest}", headers), config_digest))
    layers = iter(manifest["layers"])
    copied = 0
    found = set()
    for history in config["history"]:
        if history.get("empty_layer", False):
            continue
        layer = next(layers)
        command = history.get("created_by", "")
        prefix = next((prefix for copy, prefix in (
            ("COPY tests tests", "app/tests/"),
            ("COPY tests_utility tests_utility", "app/tests_utility/"),
        ) if command.startswith(copy + " ")), None)
        if prefix is None:
            continue
        found.add(prefix)
        digest = layer["digest"]
        data = verified(fetch(f"{REGISTRY}/blobs/{digest}", headers), digest)
        with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
            for member in archive:
                if not member.name.startswith(prefix) or not member.isfile():
                    continue
                target = (VENDOR / member.name.removeprefix("app/")).resolve()
                if not target.is_relative_to(VENDOR.resolve()):
                    raise ValueError(f"Unsafe archive path: {member.name}")
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(archive.extractfile(member).read())
                copied += 1
    if found != {"app/tests/", "app/tests_utility/"}:
        raise ValueError("Official test layers not found in image")
    print(f"Restored {copied} official files from {SOURCE['image']}@{SOURCE['digest']}")


if __name__ == "__main__":
    main()
