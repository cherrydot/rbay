#! /bin/python
import argparse
import json
import re

import requests


class Category:
    def __init__(self, code: int, name: str):
        self.code = code
        self.name = name
        self.subcategories: list[tuple[int, str]] = []

    def add_subcategory(self, code: int, name: str):
        self.subcategories.append((code, name))


class Data:
    def __init__(self, categories: list[Category], trackers: list[str]):
        self.categories = categories
        self.trackers = trackers


def get_data(mirror: str = "https://thepiratebay.org") -> Data:
    response = requests.get(mirror + "/static/main.js", allow_redirects=False)
    response.raise_for_status()
    # This is pretty brittle but oh well.
    matches = re.findall(r"category:(\d{3})[^>]*>([^<]+)<", response.text)
    flat = [(int(code), name.strip()) for code, name in matches]
    categories: list[Category] = []
    for code, name in flat:
        if code % 100 == 0:
            categories.append(Category(code, name))
        else:
            categories[-1].add_subcategory(code, name)
    trackers = re.findall(r"encodeURIComponent\('(udp://[^']+)'", response.text)
    return Data(categories, trackers)


def print_data(data: Data):
    print("Categories:")
    for category in data.categories:
        print(f"  {category.code} {category.name}")
        for code, name in category.subcategories:
            print(f"    {code} {category.name}: {name}")
    print("\nTrackers:")
    for tracker in data.trackers:
        print(f"  {tracker}")


def json_dump(data: Data):
    categories = [
        {
            "code": cat.code,
            "name": cat.name,
            "subcategories": [
                {"code": code, "name": name} for code, name in cat.subcategories
            ],
        }
        for cat in data.categories
    ]
    print(json.dumps({"categories": categories, "trackers": data.trackers}, indent=4))


def codegen(data: Data):
    gen = "//! This file was generated automatically by scraper.py.\n\n"
    flat_cats: list[tuple[int, str]] = []
    for category in data.categories:
        flat_cats.append((category.code, category.name))
        for code, name in category.subcategories:
            flat_cats.append((code, f"{category.name}: {name}"))
    gen += "/// Category codes and associated names.\n"
    gen += f"pub const CATEGORIES: [(u16, &str); {len(flat_cats)}] = [\n"
    for code, name in flat_cats:
        gen += f"    ({code}, \"{name}\"),\n"
    gen += "];\n\n"
    gen += "/// Trackers used in magnet links on thepiratebay.org.\n"
    gen += f"pub const TRACKERS: [&str; {len(data.trackers)}] = [\n"
    for tracker in data.trackers:
        gen += f"    \"{tracker}\",\n"
    gen += "];\n"
    print(gen)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Get content categories and trackers from thepiratebay.org"
    )
    parser.add_argument(
        "-m", "--mirror", default="https://thepiratebay.org", help="Mirror to use"
    )
    output = parser.add_mutually_exclusive_group()
    output.add_argument("-t", "--text", action="store_true", help="Output as text")
    output.add_argument("-j", "--json", action="store_true", help="Output as JSON")
    output.add_argument("-c", "--codegen", action="store_true", help="Output Rust code")
    args = parser.parse_args()
    data = get_data(args.mirror)
    if args.json:
        json_dump(data)
    elif args.codegen:
        codegen(data)
    else:
        print_data(data)
