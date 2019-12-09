# Denarii: Multi tenant support for SmartNICs

# Install Gurobi

Follow instructions in [Gurobi Documentation](https://www.gurobi.com/documentation/quickstart.html)
to install and setup license key for your machine.

To compile `gurobi_example.c`, use:
```
gcc -m64 -I/opt/gurobi811/linux64/include/ gurobi_example.c -L /opt/gurobi811/linux64/lib/ -lgurobi81 -lm
```
