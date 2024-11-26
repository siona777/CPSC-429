import os
import json
import statistics
import subprocess
from openai import OpenAI
import time

IMPORTANT_DIRECTORIES = ["config", "data"]
ALLOWED_LANGUAGES = {"python3"}
MAX_FILE_SIZE = 10 * 1024 * 1024  # 10 MB limit for files
ALLOWED_EXTENSIONS = {".py", ".txt"}

# Set up your OpenAI API key
client = OpenAI(
    api_key='sk-svcacct-5BdMBY7Bfu5BzGkMjNwY2OtZHDda-wE7ZU-mAKF5rbNhvtvwlQpXeDEaMMg1mb9h66vKZC5T3BlbkFJMr7ql4Gq6-yrLvMXfpdshAEkh_DoOe1r7JA6PBGbTj7OYUm35OqMsjJ_VXtB4eUZtcCXpNwA',
    organization='org-mETM9MxGgjBXBjKmv4nbS6uJ',
    project='proj_v8JiNLSbdCBK3AIO2psnuDQX',
)

def call_openai_api(prompt):
    """
    Function to call OpenAI's API and generate a response based on the prompt.
    """
    completion = client.chat.completions.create(
        model="gpt-4o",
        messages=[{"role": "user", "content": prompt}],
        stream=False
    )
    
    return completion.choices[0].message.content

def get_user_input():
    """
    Function to get the task description and working directory from the user.
    """
    task_description = input("Enter the task description: ").strip()
    working_directory = input("Enter the path to the working directory: ").strip()
    
    if not os.path.exists(working_directory):
        os.makedirs(working_directory)
        print(f"Created new working directory at {working_directory}")
    else:
        print(f"Using existing working directory at {working_directory}")
    
    return task_description, working_directory

def validate_target(target, working_directory):
    """
    Function to validate the target path.
    """
    if os.path.basename(target) != "chatgpt_code.py":
            raise ValueError("Updating non gpt file not allowed.")
    
    # Ensure the target is within the working directory
    if not target.startswith(working_directory):
        raise ValueError(f"Target '{target}' is outside the working directory.")
    
    # Check the file extension
    if os.path.splitext(target)[1] not in ALLOWED_EXTENSIONS:
        raise ValueError(f"File extension for '{target}' is not allowed.")
    
    # Check the file size if it exists
    if os.path.exists(target) and os.path.isfile(target):
        if os.path.getsize(target) > MAX_FILE_SIZE:
            raise ValueError(f"File size for '{target}' exceeds the limit of {MAX_FILE_SIZE} bytes.")

def interpret_json_command(command_json, working_directory):
    """
    Function to interpret and execute the JSON command.
    """
    command = json.loads(command_json)
    action = command.get("action")
    target = os.path.join(working_directory, command.get("target"))
    stdout, stderr = '', ''
    validate_target(target, working_directory)

    if action == "create":
        with open(target, 'w') as file:
            file.write(command.get("content", ""))
        return None, None

    elif action == "update":
        if os.path.basename(target) == "main.py":
            raise ValueError("Updating 'main.py' is not allowed.")
        
        after = command.get("after", "")

        with open(target, 'w') as file:
            file.write(after)
            
        return None, None

    elif action == "delete":
        for important_dir in IMPORTANT_DIRECTORIES:
            if os.path.commonpath([target]) == os.path.commonpath([working_directory, important_dir]):
                raise ValueError(f"Deleting important directory {important_dir}")
                                 
        if os.path.exists(target):
            os.remove(target)
        return None, None

    elif action == "execute":
        args = command.get("arguments", [])
        process = subprocess.Popen([command.get("language"), target] + args, 
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        stdout, stderr = process.communicate()
        return stdout.decode(), stderr.decode()

    else:
        raise ValueError(f"Unknown action: {action}")
    
def measure_latency(task_description, working_directory):
    start_time = time.time()
    json_command = generate_json_command(task_description, working_directory, "")
    if json_command.startswith("```"):
        json_command = json_command.split("\n")[1:-1]
        json_command = "\n".join(json_command)

    json_commands_list = json.loads(json_command)
    for command in json_commands_list:
        command_json = json.dumps(command)
        stdout, stderr = interpret_json_command(command_json, working_directory)
        if stderr:
            if len(stderr.strip()) > 0:
                task_description = analyze_logs(stdout, stderr, task_description)
    end_time = time.time()
    return end_time - start_time

def generate_json_command(task_description, working_directory, prev_code):
    """
    Function to generate a JSON command using OpenAI's API based on the task description.
    """
    # List the contents of the working directory to inform ChatGPT
    directory_contents = os.listdir(working_directory)
    if directory_contents:
        directory_listing = '\n'.join(directory_contents)
    else:
        directory_listing = 'The directory is empty.'
    
    # Prepare the prompt
    prompt = f"""
            You are to write JSON commands in strict JSON format that performs the following task:
            {task_description}
            
            The commands should work in the directory: {working_directory}
            The current contents of the directory are:
            {directory_listing}
            
            The JSON commands should strictly follow this structure:
            {{
                "action": "create|update|delete|execute",
                "target": "filename" to create, delete, or containing code to execute,
                "content": "script content here if creating",
                "before": "code to be modified from previous response if updating",
                "after": "modified code segment if updating",
                "language": "python3",
                "arguments": ["arg1", "arg2"] if executing
            }}
            
            return the JSON commands as elements in a JSON array without explanations. if you create a .py file, name it chatgpt_code.py. 
            for updates, update according to the code you generated in the previous response and assume the same input as the previous query.
            """
    if prev_code:
        prompt = prompt + "\n the previous code was \n" + prev_code
    json_command = call_openai_api(prompt)
    return json_command


def analyze_logs(stdout, stderr, task_description):
    """
    Function to analyze the execution logs using OpenAI's API to determine success.
    """
    # Prepare the prompt for analysis
    prompt = f"""
    The following execution logs are from a JSON command intended to perform the task:
    {task_description}
    
    Standard Output (STDOUT):
    {stdout}
    
    Standard Error (STDERR):
    {stderr}
    
    Based on these logs, did the command complete the task successfully? If not, what errors occurred, and how can they be fixed?
    
    Revise the original task description so that it would lead to a successful execution.
    
    Only provide the revised task description without any additional explanations.
    """
    
    # Call OpenAI API to analyze the logs
    analysis = call_openai_api(prompt)
    return analysis

def timing_main(task_description, working_directory):
    interface_times = []

    for _ in range(5):
        interface_times.append(measure_latency(task_description, working_directory))
    avg_interface_latency = statistics.mean(interface_times)
    print(f"Interface Design Latency (Average): {avg_interface_latency:.4f} seconds")

def main():
    """
    Main function to run the CODA workflow.
    """    
    task_description, working_directory = get_user_input()

    prev_code = ""

    while(True):
        json_command = generate_json_command(task_description, working_directory, prev_code)
        
        # if it starts with ```, remove the first and the last line
        if json_command.startswith("```"):
            json_command = json_command.split("\n")[1:-1]
            json_command = "\n".join(json_command)

        print("\nGenerated JSON Commands Array:\n")
        print(json_command)
        
        try:
            # Parse the JSON array of commands
            json_commands_list = json.loads(json_command)
            
            for command in json_commands_list:
                command_json = json.dumps(command)  # Convert back to JSON string
                # Interpret each JSON command and execute it
                stdout, stderr = interpret_json_command(command_json, working_directory)
                if stdout or stderr:  # Only analyze logs if there is output or error
                    if len(stderr.strip()) > 0:
                        print("STDERR:\n", stderr)
                        task_description = analyze_logs(stdout, stderr, task_description)
                        print("\nRevised description:\n")
                        print(task_description)
            
            # Ask for a follow-up task
            print("\nTask completed successfully.")
            follow_up = input("\nEnter a follow-up task description (or press Enter to exit): ").strip()
            if not follow_up:
                print("Exiting CODA.")
                return
            else:
                task_description = follow_up
                target = os.path.join(working_directory, "chatgpt_code.py")
                with open(target, 'r') as file:
                    prev_code = file.read()
        except Exception as e:
            print(f"An error occurred: {e}")

if __name__ == "__main__":
    main()