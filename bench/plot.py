import matplotlib
matplotlib.use('TkAgg')  # or 'Qt5Agg' or any other interactive backend
import matplotlib.pyplot as plt
import pandas as pd

def plot_multiple_methods(time, count,names):
    for i in range(len(names)):
        plt.plot(count[i], time[i], label=names[i]) 
    plt.xlabel('Count')
    plt.ylabel('Time')
    plt.title('Multiple Methods Comparison')
    plt.legend()
    plt.show()

df = pd.read_csv("results.csv")
df.columns = ["size_fasta", "min_len", "max_gap", "mismatches", "cpp_timing", "rust_timing"]
df1 = df[df['size_fasta'] == 100000]

# Max gap
df2 = df1[(df1["min_len"] == 10) & (df1["mismatches"] == 2)]

# Now make a plot with max_gap on the x and two curves: cpp_timing and rust_timing 

# Extract relevant columns for plotting
max_gap_values = df2["max_gap"]
cpp_timing_values = df2["cpp_timing"]
rust_timing_values = df2["rust_timing"]

# Plotting
plt.plot(max_gap_values, cpp_timing_values, label="cpp_timing")
plt.plot(max_gap_values, rust_timing_values, label="rust_timing")

# Set labels and title
plt.xlabel("Max Gap")
plt.ylabel("Timing")
plt.title("Timing Comparison with Max Gap")

# Show legend
plt.legend()

# Show the plot
plt.show()

# print(df2)