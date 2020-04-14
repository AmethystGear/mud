import java.io.*;
import java.util.*;

// class that essentially holds a map from Strings -> Objects
// allows storing and reading of files.
// file format should be of the form:
// /begin/
// {property-name}: {property-value}
// .
// .
// /end/
// for each Stats object that will be parsed from the file.
class Stats {
    private HashMap<String, String> types;
    private HashMap<String, Object> stats;

    // create a Stats object with nothing in it.
    public Stats() {
        types = new HashMap<String, String>();
        stats = new HashMap<String, Object>();
    }

    // creates a new stats class from a Scanner.
    // throws IllegalArgumentException if the Scanner has invalid input.
    public Stats (Scanner scan) throws IllegalArgumentException {
        stats = new HashMap<>();
        while(scan.hasNextLine()) {
            String line = scan.nextLine();
            Scanner lineScan = new Scanner(scan.nextLine());
            if(line.trim().equals("/end/")) {
                lineScan.close();
                return;
            }
            else if(lineScan.hasNext()) {
                String type = lineScan.next();
                String varname = lineScan.next().replace('_', ' ').replace('-', ' ');
                Object value;
                if (type.equals("boolean")) {
                    if(!lineScan.hasNextInt()) {
                        lineScan.close();
                        throw new IllegalArgumentException("variable of type boolean does not contain a boolean!");
                    }
                    value = lineScan.nextBoolean();
                } else if (type.equals("int")) {
                    if(!lineScan.hasNextInt()) {
                        lineScan.close();
                        throw new IllegalArgumentException("variable of type int does not contain an int!");
                    }
                    value = lineScan.nextInt();
                } else if (type.equals("int[]")) {
                    String[] input = ScannerUtils.getRemainingInputAsStringArray(lineScan);
                    int[] values = new int[input.length];
                    for(int i = 0; i < values.length; i++) {
                        try {
                            values[i] = Integer.parseInt(input[i]);
                        } catch(NumberFormatException e) {
                            lineScan.close();
                            throw new IllegalArgumentException("variable of type int[] contains non-int values!");
                        }
                    }
                    value = values;
                } else if (type.equals("String")) {
                    value = ScannerUtils.getRemainingInputAsString(lineScan);
                } else if (type.equals("String[]")) {
                    value = ScannerUtils.getRemainingInputAsStringArray(lineScan);
                } else {
                    lineScan.close();
                    throw new IllegalArgumentException("type not recognized!");
                }
                types.put(varname, type);
                stats.put(varname, value);
            }
            lineScan.close();
        }
    }

    // this constructor is for the clone method only, otherwise our RI could be violated by bad inputs.
    private Stats (HashMap<String, String> types, HashMap<String, Object> stats) {
        this.types = types;
        this.stats = stats;
    }

    // create a deep copy of this stats object and return it.
    public Stats clone() {
        HashMap<String, String> typesCopy = new HashMap<>();
        HashMap<String, Object> statsCopy = new HashMap<>();
        for(Map.Entry<String, String> e : types.entrySet()) {
            typesCopy.put(e.getKey(), e.getValue());
        }
        for(String key : stats.keySet()) {
            statsCopy.put(key, get(key));
        }
        return new Stats(typesCopy, statsCopy);
    }

    // save this stats class to a file by appending it to the end of that file.
    public void saveTo(String file) throws FileNotFoundException {
        PrintWriter writer = new PrintWriter(new FileOutputStream(new File(file)), true);
        writer.append("\n/begin/\n");
        for(Map.Entry<String, Object> e : stats.entrySet()) {
            writer.append(types.get(e.getKey()));
            writer.append(" ");
            writer.append(e.getKey().replace(' ', '-'));
            writer.append(getStringRep(e.getValue(), true));
            writer.append("\n");
        }
        writer.append("/end/\n");
        writer.close();
    }

    // check if this stat has a variable with the given name.
    public boolean hasVariable(String name) {
        return stats.containsKey(name);
    }

    // return the object corresponding to the variable name.
    public Object get(String name) {
        if(!stats.containsKey(name)) {
            throw new IllegalArgumentException("that variable doesn't exist!");
        }
        // return a copy if returning an array.
        if(stats.get(name) instanceof int[] || stats.get(name) instanceof String[]) {
            Object[] arr = (Object[])stats.get(name);
            Object[] copy = new Object[arr.length];
            for(int i = 0; i < arr.length; i++) {
                copy[i] = arr[i];
            }
            return copy;
        }
        return stats.get(name);
    }

    // set the variable with the given name to the given value.
    // throws IllegalArgumentException if the type of this variable is not valid
    // (see isValidType)
    public void set(String name, Object value) {
        if(!isValidType(value)) {
            throw new IllegalArgumentException("the type provided is invalid!");
        }
        types.put(name, value.getClass().getName());
        stats.put(name, value);
    }

    // check if provided object is one of the types we can parse.
    private boolean isValidType(Object o) {
        return o instanceof Boolean || 
               o instanceof Integer ||
               o instanceof String ||
               o instanceof int[] ||
               o instanceof String[];
    }

    // string rep of a value. 
    private String getStringRep(Object o, boolean replaceSpaces) {        
        if(o instanceof int[] || o instanceof String[]) {
            Object[] arr = (Object[])o;
            StringBuilder s = new StringBuilder();
            for(int i = 0; i < arr.length; i++) {
                if(replaceSpaces) {
                    s.append(arr[i].toString().replace(' ', '-'));
                } else {
                    s.append(arr[i].toString());
                }
                if(i != arr.length - 1) {
                    s.append(" ");
                }
            }
            return s.toString();
        } else {
            if(replaceSpaces) {
                return o.toString().replace(' ', '-');
            } else {
                return o.toString();
            }
        }
    }

    // string rep of all the stats.
    public String toString() {
        StringBuilder s = new StringBuilder();
        for(Map.Entry<String, Object> e : stats.entrySet()) {
            s.append(e.getKey());
            s.append(": ");
            s.append(getStringRep(e.getValue(), false));
            s.append("\n");
        }
        return s.toString();
    }
}
