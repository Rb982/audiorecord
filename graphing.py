import sys
import matplotlib.pyplot as plt
import matplotlib
filename = sys.argv[1]
data_x = []
data_y = []
file = open(filename)
for x in file:
    try:
        if(len(x)>1):
            xs = x.split(" ")
            data_x.append(xs[0])
            data_y.append(xs[1])
       # data.append(x[0:len(x)-1])
    except(ValueError):
        pass
#print(data)
xaxis=[*range(0, len(data_x))]
fig, ax = plt.subplots()
ax.plot(xaxis, data_y)
ax.plot(xaxis, data_x)
#matplotlib.use("macosx")
#fig, ax = plt.subplots()  # Create a figure containing a single axes.
#ax.plot([1, 2, 3, 4], [1, 4, 2, 3])  # Plot some data on the axes.
plt.show()