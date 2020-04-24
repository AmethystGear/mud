import os
import glob
files = glob.glob('server/**/*.java', recursive=True)
class_list = open("temp-classes.txt", "w")
for file in files:
    class_list.write(file + " ")
class_list.close()

os.system("javac @temp-classes.txt")
os.remove("temp-classes.txt")
