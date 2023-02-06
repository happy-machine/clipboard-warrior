import time
import random
import string

def run_model(**kwargs):
    print("running model")
    time.sleep(random.randint(0,9) * 0.005)
    # simulate variable response time
    return f"{kwargs['prompt']} : {''.join(random.choice(string.digits) for i in range(10))}"

def tokenizer(string):
    return string.split()