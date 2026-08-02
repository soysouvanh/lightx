import base64
import urllib.request
import os

files = ['architecture_fr', 'aop_fr', 'architecture_en', 'aop_en', 'context_fr', 'context_en']

for f in files:
    with open(f"assets/{f}.mmd", "r") as mmd:
        data = mmd.read().encode("utf-8")
        b64 = base64.b64encode(data).decode("ascii")
        url = f"https://mermaid.ink/svg/{b64}?bgColor=ffffff"
        try:
            req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
            with urllib.request.urlopen(req) as response:
                svg = response.read().decode('utf-8')
                with open(f"assets/{f}.svg", "w") as svg_file:
                    svg_file.write(svg)
            print(f"Generated assets/{f}.svg")
        except Exception as e:
            print(f"Failed to generate {f}.svg: {e}")
