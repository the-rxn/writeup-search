### Main idea here was to use ThreadPoolExecutor to make download writeups superfast, but as it turns out 
### CTFtime can't handle so many requests, so i just made it sequential and 1 thread 
###
### sad.
###
### However you can choose what type of tactic you want to use

import requests
import sys
from concurrent.futures import ThreadPoolExecutor

# generate urls for pools
url_template = "https://ctftime.org/writeup/{i}"
filename_template = "writeups/{i}.html"
urls_and_files = [(url_template.format(i=i), filename_template.format(i=i)) for i in range(1, 38700)]

# user-agent header to not get 403 from nginx
headers = {'user-agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36'}

# inp = (url, filename)
def download_html(inp: (str, str)):
    (url, filename) = (inp[0], inp[1])
    writeup_str = f"{filename.replace('.html', '').split('/')[1]}"
    response = requests.get(url, headers=headers)
    # let's not try to ddos ctftime and retry
    if response.status_code == 503:
        print(f"Got 503 code on writeup #{writeup_str}")
        sleep(2)
        download_html(inp)
    if response.status_code != 200:
        print(f"Error occured: {response.status_code} when trying to download writeup #{writeup_str}")
        return
    with open(filename, "wb") as file:
        file.write(response.content)
    print(f"Successfully downloaded writeup #{writeup_str}")

def main():
    if len(sys.argv) > 2:
        print("USAGE:\nSingle thread: python scripts/download_writeups\nMulti thread: python scripts/download_writeups multi") 
        exit(0)
    if len(sys.argv) == 2: 
        if sys.argv[1] == "multi":
            print("Starting downloading writeups from CTFTime concurrently")
            with ThreadPoolExecutor(max_workers=5) as executor:
                executor.map(download_html, urls_and_files)
        else:
            print("USAGE:\nSingle thread: python scripts/download_writeups\nMulti thread: python scripts/download_writeups multi") 
            exit(0)
    else:
        print("Starting downloading writeups from CTFTime using one thread")
        for inp in urls_and_files:
            download_html(inp)

if __name__ == "__main__":
    main()