# A docker file to be ran before `marqo` in order to download already done embeddings of 38716.
# If you want to use newer writeups, you need to pull them into json from id 38176

# FROM python:latest 
# RUN pip install gdown
#
# RUN echo "Downloading already embedded writeups..."
# RUN gdown --id 1YcBL-490B7u3K5dbgUYQUuRF8q24VvW4 -O ./tmp.tar.gz
#
# RUN echo "Extracting data into shared volume..."
# RUN mkdir /vespa
# RUN tar --same-owner -xvf ./tmp.tar.gz --directory=/vespa
#
# RUN echo "All done. Now building marqo..."
# RUN ls /vespa
#
# CMD ["/bin/bash"]

#
FROM marqoai/marqo:2.2.0

RUN echo "Installing gdown..."
RUN pip install gdown

RUN echo "Downloading already embedded writeups..."
RUN gdown --id 12pusANQqEo-GQsgIs8JEmIR6KAHhqLbE -O ./back.tar.gz

COPY ./is_vespa_ready.sh .
COPY ./create_writeups_index.py .
RUN chmod +x ./is_vespa_ready.sh 

RUN echo "Extracting data into /app/back..."
RUN tar --same-owner -xvf ./back.tar.gz 

RUN echo "Extracted data. Now adding predone writeup embeddings into vespa..."

ENTRYPOINT ["/bin/bash", "-c", "./run_marqo.sh & ./is_vespa_ready.sh"]




