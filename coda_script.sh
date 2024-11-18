#!/bin/bash

# Change to the specified directory
cd /home/accts/st943/coda

# Clear the status_report.txt file
: > status_report.txt

# Read URLs from urls.txt and process each one
while read -r url; do
    if [[ -n "$url" ]]; then
        # Measure the response time and status code for each URL
        response=$(curl -o /dev/null -s -w "%{http_code} %{time_total}\n" "$url")
        status_code=$(echo $response | awk '{print $1}')
        response_time=$(echo $response | awk '{print $2}')

        # Write the URL, status code, and response time to status_report.txt
        echo "$url - Status: $status_code - Response Time: ${response_time}s" >> status_report.txt
    fi
done < urls.txt