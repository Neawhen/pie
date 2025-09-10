import json
import os
from pathlib import Path

def append_log(log_path, data):
    """
    Append data to a JSON log file.

    Args:
        log_path (str): Path to the log file
        data (dict): Data to append to the log
    """
    # Ensure the logs directory exists
    log_dir = Path(log_path).parent
    log_dir.mkdir(parents=True, exist_ok=True)

    # Read existing data if file exists
    existing_data = []
    if os.path.exists(log_path):
        try:
            with open(log_path, 'r', encoding='utf-8') as f:
                existing_data = json.load(f)
                if not isinstance(existing_data, list):
                    existing_data = [existing_data]
        except (json.JSONDecodeError, FileNotFoundError):
            existing_data = []

    # Append new data
    existing_data.append(data)

    # Write back to file
    with open(log_path, 'w', encoding='utf-8') as f:
        json.dump(existing_data, f, indent=2, ensure_ascii=False)

