import os
import statistics
import subprocess
import psutil  # For reading system information
from openai import OpenAI
import time

IMPORTANT_DIRECTORIES = ["config", "data"]
ALLOWED_LANGUAGES = {"python3"}
MAX_FILE_SIZE = 10 * 1024 * 1024  # 10 MB limit for files
ALLOWED_EXTENSIONS = {".py", ".txt", ".rs"}

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
    if os.path.basename(target) != "chatgpt_code.rs":
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

def execute_code(language, target, args=[]):
    print("executing code")
    start_time = time.time()
    compile_process = subprocess.Popen([language, target] + args,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    compile_stdout, compile_stderr = compile_process.communicate()

    if compile_process.returncode != 0:
        return "", compile_stderr.decode(), None
    else:
        # Run the compiled program
        executable = os.path.join(os.path.dirname(target), "chatgpt_code")
        run_process = subprocess.Popen([executable],
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        run_stdout, run_stderr = run_process.communicate()

    end_time = time.time()
    return run_stdout.decode(), run_stderr.decode(), end_time - start_time

def fix_stderr(stderr, stdout, task_description, working_directory):
    while(stderr):
        if len(stderr.strip()) > 0:
            print("fail\n", stderr)
            task_description = analyze_logs(stdout, stderr, task_description) + "\n return only code nothing else. You are only allowed to use built-in Rust functions and functions Rust standard library. External dependencies are unallowed"
            print("failed task desc\n", task_description)
            fixed_code = call_openai_api(task_description)
            fixed_code = strip_markdown_top(fixed_code)
            target = os.path.join(working_directory, "chatgpt_code.rs")
            with open(target, 'w') as file:
                file.write(fixed_code)
            stdout, stderr, latency = execute_code("rustc", target)
    return latency
                
def measure_code_latency(working_directory):
    print("measuring code latency")
    target = os.path.join(working_directory, "chatgpt_code.rs")
    return execute_code("rustc", target)

def measure_latency(task_description, working_directory, hardware):
    rust_code = generate_rust_code(task_description, working_directory, "", hardware)
    write_to_file(working_directory, rust_code)
    target = os.path.join(working_directory, "chatgpt_code.rs")
    stdout, stderr, new_latency = execute_code("rustc", target, [])
    if stderr:
        new_latency = fix_stderr(stderr, stdout, task_description, working_directory)
    return rust_code, new_latency

def strip_markdown_top(code):
    if code.startswith("```"):
        code = code.split("\n")[1:-1]
        code = "\n".join(code)
    if code.endswith("```"):
        code = code.rsplit("\n", 1)[0]
    return code

def generate_rust_code(task_description, working_directory, prev_code, hardware):
    """
    Function to generate a JSON command using OpenAI's API based on the task description.
    """
    prompt = f"""
            Write rust code that performs the following task:
            {task_description}
            when writing code use all of the hardware possible for fastest latency.
            current system hardware: {hardware}
            only output code that can be compiled, no explanations.
            """
    # if prev_code:
    #     prompt = prompt + "\n the previous code was \n" + prev_code
    code = call_openai_api(prompt)
    return strip_markdown_top(code)

def analyze_logs(stdout, stderr, task_description):
    """
    Function to analyze the execution logs using OpenAI's API to determine success.
    """
    prompt = f"""
    The following execution logs are from rust code intended to perform the task:
    {task_description}
    Standard Output (STDOUT):
    {stdout}
    Standard Error (STDERR):
    {stderr}
    Based on these logs, what errors occurred, and how can they be fixed?
    Revise the original task description so that it would lead to a successful execution.
    Only provide the revised task description without any additional explanations.
    """
    analysis = call_openai_api(prompt)
    return analysis

def read_system_info():
    """
    Function to read and return system information.
    """
    cpu_info = psutil.cpu_freq()
    system_info = {
        "cpu_count": psutil.cpu_count(logical=False),
        "cpu_logical_count": psutil.cpu_count(logical=True),
        "cpu_freq_max": cpu_info.max if cpu_info else "N/A",
        "cpu_freq_min": cpu_info.min if cpu_info else "N/A",
        "memory_total": psutil.virtual_memory().total
    }
    return system_info

def write_to_file(working_directory, code):
    target = os.path.join(working_directory, "chatgpt_code.rs")
    with open(target, 'w') as file:
        file.write(code)

def generate_and_apply_optimization(system_hardware, working_directory, current_latency):
    with open('chatgpt_code.rs', 'r') as f:
        prev_code = f.read()
        
    hypothesis = generate_optimization_hypothesis(prev_code, current_latency, system_hardware)
    print(f"Generated Hypothesis: {hypothesis}")
    
    # Update the code based on the hypothesis
    updated_code = call_openai_api(f"output code and no explanations. You are only allowed to use built-in Rust functions and functions Rust standard library. External dependencies are unallowed. Update the following rust code to get better latency based on this hypothesis: {hypothesis}\n\nCode:\n{prev_code}\n Optimize for system hardware {system_hardware}\n")
    updated_code = strip_markdown_top(updated_code)
    write_to_file(working_directory, updated_code)
    return hypothesis, updated_code

def generate_optimization_hypothesis(code, latency, system_hw):
    """
    Generate a hypothesis for optimization based on current and previous latency.
    """
    prompt = f"""
            Generate a hypothesis for improving the latency for the following code – it is {latency} right now. code: \n{code}\n the system hardware is {system_hw}. You are only allowed to use built-in Rust functions and functions Rust standard library. External dependencies are unallowed.
    """
    hypothesis = call_openai_api(prompt)
    return hypothesis

def main():
    """
    Main function to run the CODA workflow.
    """
    system_info = read_system_info()
    print(f"System Information: {system_info}")
    task_description, working_directory = get_user_input()

    previous_latency = None
    max_iterations = 100
    prev_code, current_latency = measure_latency(task_description, working_directory, system_info)
    print(f"Iteration 1: Latency = {current_latency:.4f} seconds")

    for i in range(max_iterations):
        task_description, updated_code = generate_and_apply_optimization(system_info, working_directory, current_latency)
        stdout, stderr, new_latency = measure_code_latency(working_directory)
        
        if stderr:
            new_latency = fix_stderr(stderr, stdout, task_description, working_directory)

        print(f"Iteration {i + 2}: Latency = {new_latency:.4f} seconds")
        
        if new_latency < current_latency:
            print("Hypothesis accepted. Keeping the updated code.")
            prev_code = updated_code
            current_latency = new_latency
        else:
            print("Hypothesis rejected. Reverting to previous code.")
            # Revert to previous code
            target = os.path.join(working_directory, "chatgpt_code.rs")
            with open(target, 'w') as file:
                file.write(prev_code)

if __name__ == "__main__":
    main()
