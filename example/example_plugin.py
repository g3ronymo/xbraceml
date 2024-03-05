#!/usr/bin/env python3
import sys

def main():
    SEP = "\r\n\r\n"
    elements = (
            "1h",
            "2h",
    )

    if "elements" in sys.argv:
        print(" ".join(elements), end="")
    else:
        #data[0] == name, data[1] == attr, data[2] == content
        name, atrr, content = sys.stdin.read().split(SEP, maxsplit=2)
        if name == "1h":
            print(f'<h1 style="color:#ff0000; text-decoration: underline;">{content}</h1>', end="")
        elif name == "2h":
            print(f'<h2 style="color:#800000; text-decoration: underline;">{content}</h2>', end="")

if __name__ == "__main__":
    main()


