#!/usr/bin/env python3

import requests

def check_urls(file_path):
    try:
        with open(file_path, "r") as url_file:
            urls = url_file.readlines()
        with open("status_report.txt", "w") as report_file:
            for url in urls:
                url = url.strip()
                try:
                    response = requests.get(url)
                    status = "Online" if response.status_code == 200 else "Offline"
                except requests.RequestException:
                    status = "Offline"
                report_file.write(f"{url} {status}\n")
    except FileNotFoundError:
        print(f"Could not find file: {file_path}")

if __name__ == "__main__":
    check_urls("urls.txt")
