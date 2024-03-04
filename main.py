import marqo
import json
import pprint
import argparse

MAX_BATCH_SIZE=128

def read_writeups_json(filepath: str):
    with open(filepath, "r") as file:
        return json.load(file)

def main():
    parser = argparse.ArgumentParser(
        prog='uploader',
        description='uploads writeups from json to a specified marqo instance'
    )
    parser.add_argument('filename')
    parser.add_argument('-u', '--url',required=True)
    args = parser.parse_args()



    mq = marqo.Client(url=parser.url)
    INDEX_NAME = "writeups"
    BATCH_SIZE=4
    STOPPED_AT=384

    mq.create_index(INDEX_NAME)
    documents = read_writeups_json(parser.filename)
    n = BATCH_SIZE
    documents_batched= [documents[i * n:(i + 1) * n] for i in range((len(documents) + n - 1) // n )]
    num = 0
    for i in range(0, len(documents_batched)):
        res = mq.index(INDEX_NAME).add_documents(documents_batched[i], tensor_fields=['title', 'description', 'tags'])
        num += BATCH_SIZE
        print(res)
        print(f"Loaded {num} writeups into embedding")

    results = mq.index(INDEX_NAME).search(
        q="What to do if variable names are jumbled (java)?", limit=1
    )
    pprint.pprint(results)

if __name__ == '__main__':
    main()
