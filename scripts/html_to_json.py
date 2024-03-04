from bs4 import BeautifulSoup
from dataclasses import dataclass
import json
import attrs

TAG_DIV_SELECTOR="span7 > p:nth-child(1)"
TAG_SELECTOR = "span.label"
AUTHOR_SELECTOR ="div.page-header:nth-child(1) > a:nth-child(2)"
TEAM_SELECTOR ="div.page-header:nth-child(1) > a:nth-child(3)"
EVENT_SELECTOR =".breadcrumb > li:nth-child(3) > a:nth-child(1)"
TITLE_SELECTOR ="html body div.container div.page-header h2"
ORIG_WRITEUP_SELECTOR ="div.well:nth-child(2) > a:nth-child(1)"
BODY_WRITEUP_SELECTOR ="#id_description > p:nth-child(1) > a:nth-child(1)"
BODY_SELECTOR = "#id_description"

@attrs.frozen
class Writeup:
    author: str
    tags: list[str]
    team: str
    event: str
    title: str
    body: str

def _parse(soup, selector):
    el = soup.select_one(selector)
    if not el:
        return ""
    else:
        return el.text

def _parse_tags(soup, selector):
    el = soup.select(selector)
    if not el:
        return ""
    else:
        # print(el)
        return [e.text for e in el]

def parse_html(contents: str) -> Writeup:
    soup = BeautifulSoup(contents, features="html.parser")
    author = _parse(soup,AUTHOR_SELECTOR)
    tags = _parse_tags(soup,TAG_SELECTOR)
    team = _parse(soup,TEAM_SELECTOR)
    event = _parse(soup,EVENT_SELECTOR)
    title = _parse(soup,TITLE_SELECTOR)
    body = _parse(soup,BODY_SELECTOR)
    return Writeup(author, tags, team, event, title, body)
 
def convert_writeup_dir(path_to_dir, output_json_path): 
    paths = path_to_dir
    arr = []
    for path in paths:
        with open(path, "r") as file:
            contents = file.read()
            wr = parse_html(contents)
            if wr.body != "":
                # jsonStr = json.dumps(wr.__dict__)
                # arr.append(jsonStr)
                arr.append(wr)
    with open(output_json_path, "w") as j:
        json.dump(arr, j, default=attrs.asdict)

path_template = "../writeups/{i}.html"
writeups_dir = "../writeups"
json_output = "../writeups_py.json"
from os import listdir
from os.path import isfile, join
onlyfiles = [f for f in listdir(writeups_dir) if isfile(join(writeups_dir, f))]
full_path = [writeups_dir + "/" + f for f in onlyfiles]

convert_writeup_dir(full_path, json_output)