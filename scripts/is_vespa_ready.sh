#!/bin/bash
while :
do
	if [ $(vespa status | grep -ic "is ready") -eq 1 ] 
	then
		# need this 
		pip3 install marqo
		sleep 200
		python3 create_writeups_index.py
		echo "VESPA READY!!! Feeding now..."
		vespa feed back
		echo "Done feeding"
		exit 0
	else
		echo "Vespa not ready... Waiting..."
		sleep 2
	fi
done

