from pathlib import Path
import hashlib
import xml.etree.ElementTree as ET

NS = "http://www.w3.org/2000/svg"
ET.register_namespace("", NS)
assets = Path(__file__).resolve().parent
source_path = assets / "arterm-before-after.svg"
source = ET.fromstring(source_path.read_bytes())
cover = ET.Element(f"{{{NS}}}svg", {
    "width": "1200", "height": "630", "viewBox": "0 0 1200 630",
    "role": "img", "aria-labelledby": "cover-title cover-description",
})
ET.SubElement(cover, f"{{{NS}}}title", {"id": "cover-title"}).text = "arTerm persistent remote Windows terminal"
ET.SubElement(cover, f"{{{NS}}}desc", {"id": "cover-description"}).text = (
    "Workflow illustration: work continues after client loss on a running logged-in Windows host; "
    "return to the same shell and catch up on bounded output with possible gaps. "
    "Windows on both ends. No host reboot or logoff recovery. Not execution evidence."
)
ET.SubElement(cover, f"{{{NS}}}metadata").text = (
    "Derived from arterm-before-after.svg; source SHA-256 "
    + hashlib.sha256(source_path.read_bytes()).hexdigest()
    + ". Edit the shared hero and this composer, then regenerate this SVG."
)
ET.SubElement(cover, f"{{{NS}}}rect", {"width": "1200", "height": "630", "fill": "#ffffff"})
labels = ET.SubElement(cover, f"{{{NS}}}g", {
    "font-family": "Segoe UI, Arial, sans-serif", "fill": "#172554", "text-anchor": "middle",
})
ET.SubElement(labels, f"{{{NS}}}text", {
    "x": "600", "y": "49", "font-size": "38", "font-weight": "700",
}).text = "arTerm — Persistent remote Windows terminal"
ET.SubElement(labels, f"{{{NS}}}text", {
    "x": "600", "y": "81", "font-size": "24",
}).text = "Workflow illustration"
hero = ET.SubElement(cover, f"{{{NS}}}g", {"transform": "translate(0 94)"})
for element in source:
    if element.tag in (f"{{{NS}}}defs", f"{{{NS}}}g"):
        hero.append(element)
ET.SubElement(labels, f"{{{NS}}}text", {
    "x": "600", "y": "609", "font-size": "28",
}).text = "Windows on both ends · No host reboot/logoff recovery"
ET.indent(cover, space="  ")
output = assets / "arterm-github-cover.svg"
output.write_text(ET.tostring(cover, encoding="unicode") + "\n", encoding="utf-8")
print(f"Derived {output.name} from {source_path.name}; 1200x630.")
