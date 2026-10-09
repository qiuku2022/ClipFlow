import sys

from clipflow_worker.pipe_client import PipeClient


def main():
    if len(sys.argv) < 2:
        print("Usage: python e2e_worker.py <pipe_name>")
        sys.exit(1)
        
    pipe_name = sys.argv[1]
    client = PipeClient(pipe_name)
    
    # Try connecting with timeout
    client.connect(timeout=5.0)
    
    # Read request
    req = client.read_response()
    if req is None:
        sys.exit(2)
        
    # Simply echo back the parameters with a result
    method = req.get("method")
    msg_id = req.get("id")
    params = req.get("params", {})
    
    if method == "echo":
        result = {"echoed": params}
        client.f.write(client.encode_request("response", result, msg_id=msg_id))
        client.f.flush()
    else:
        sys.exit(3)
        
if __name__ == "__main__":
    main()
