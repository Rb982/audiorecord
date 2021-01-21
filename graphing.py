import sys
import matplotlib.pyplot as plt
import matplotlib
filename = sys.argv[1]
data = []
file = open(filename)
for x in file:
    if(len(x)>1):
        data.append(x[0:len(x)-1])
#print(data)
xaxis=[*range(0, len(data))]
fig, ax = plt.subplots()
ax.plot(xaxis, data)
#matplotlib.use("macosx")
#fig, ax = plt.subplots()  # Create a figure containing a single axes.
#ax.plot([1, 2, 3, 4], [1, 4, 2, 3])  # Plot some data on the axes.
plt.show()