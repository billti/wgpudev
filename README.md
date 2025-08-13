# WebGPU based quantum state vector simulator

## TODO

- Run an actual quantum circuit and verify the results for a simple Bell Pair
- Figure out to do logging and capture on large scale simulations
- Create the buffers for the state vector and operations list
- Dispatch workgroups for operations in a loop
- Run a quantum kernel that can do the math
- Be able to fetch and display the results

## Next steps

How to represent a list of operations in Rust that can populate the GPU op list.

A vector of some struct seems natural
bytemuck is the crate to reinterpret bytes of things
Need to figure out how to override the workgroup size and the op index for each dispatch.
