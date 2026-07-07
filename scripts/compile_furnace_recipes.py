import json
import sys
from pathlib import Path

if len(sys.argv) < 2:
    print("Usage: python compile_furnace_recipes.py <path_to_recipes_directory>")
    sys.exit(1)

target = Path(sys.argv[1])
final = {}

if not target.exists() or not target.is_dir():
    print(f"Error: {target} is not a valid directory.")
    sys.exit(1)

for file in target.glob("*.json"):
    with open(file, "r") as f:
        data = json.load(f)

        if (
            data["type"] != "minecraft:smoking"
            and data["type"] != "minecraft:blasting"
            and data["type"] != "minecraft:smelting"
        ):
            continue

        if data["type"] not in final:
            final[data["type"]] = {}

        if isinstance(data["ingredient"], list):
            for ingredient in data["ingredient"]:
                final[data["type"]][ingredient] = data["result"]["id"]
        else:
            final[data["type"]][data["ingredient"]] = data["result"]["id"]

with open("furnace_recipes.json", "w") as f:
    f.write(json.dumps(final, indent=4))
