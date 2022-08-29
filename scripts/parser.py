import os
import ast
import multiprocessing.pool


def parse(filename: str) -> None:
    with open(filename, "rb") as fp:
        contents = fp.read()
    try:
        tree = ast.parse(contents, filename)
    except:
        print(f"Failed to parse: {filename}")


if __name__ == "__main__":
    all_files = []
    for root, dirs, files in os.walk("resources/test/cpython"):
        for filename in files:
            if filename.endswith(".py"):
                all_files.append(os.path.join(root, filename))

    with multiprocessing.Pool(processes=multiprocessing.cpu_count()) as p:
        print(p.map(parse, all_files))
