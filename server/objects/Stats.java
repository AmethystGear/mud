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
    private HashSet<String> properties;

    // create a Stats object with nothing in it.
    public Stats() {
        types = new HashMap<String, String>();
        stats = new HashMap<String, Object>();
        properties = new HashSet<String>();
    }

    // creates a new stats class from a Scanner.
    // throws IllegalArgumentException if the Scanner has invalid input.
    public Stats (Scanner scan) throws IllegalArgumentException {
        types = new HashMap<>();
        stats = new HashMap<>();
        while(scan.hasNextLine()) {
            String line = scan.nextLine();
            if(line.startsWith("#")) { //ignore comments
                continue;
            }
            Scanner lineScan = new Scanner(line);
            if(lineScan.hasNext()) {
                String type = lineScan.next();
                if(type.equals("/begin/")) {
                    continue;
                }
                if(type.equals("/end/")) {
                    break;
                }

                // properties are a special case because they don't have values         
                if(type.equals("prop")) {
                    if(!lineScan.hasNext()) {
                        throw new IllegalArgumentException("property doesn't have a name!");
                    }
                    properties.add(lineScan.next().replace('-', ' ').replace('_', ' '));
                    continue;
                }

                if(!lineScan.hasNext()) {
                    throw new IllegalArgumentException("variable doesn't have a name!");
                }

                String varname = lineScan.next();
                Object value;
                if (type.equals("int")) {
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
                } else if (type.equals("LongString")) {
                    value = ScannerUtils.getInputTillEnd(scan);
                    types.put(varname.replace('-', ' '), type);
                    stats.put(varname.replace('-', ' '), value);
                    break;
                } else if (type.equals("double")) {
                    if(!lineScan.hasNextDouble()) {
                        lineScan.close();
                        throw new IllegalArgumentException("variable of type double does not contain a double!");
                    }
                    value = lineScan.nextDouble();
                } else {
                    lineScan.close();
                    System.out.println(type);
                    throw new IllegalArgumentException("type not recognized!");
                }
                types.put(varname.replace('-', ' '), type);
                stats.put(varname.replace('-', ' '), value);
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

    public boolean hasProperty(String prop) {
        return properties.contains(prop);
    }

    public void addProperty(String prop) {
        properties.add(prop);
    }

    // save this stats class to a file by appending it to the end of that file.
    public void saveTo(String file) throws FileNotFoundException {
        PrintWriter writer = new PrintWriter(new FileOutputStream(new File(file)), true);
        writer.append("\n/begin/\n");
        for(String prop : properties) {
            writer.append("prop ");
            writer.append(prop.replace(' ', '-'));
            writer.append("\n");
        }
        String longStringVarName = null;
        StringBuilder longString = null;
        for(Map.Entry<String, Object> e : stats.entrySet()) {
            if(types.get(e.getKey()).equals("LongString")) {
                longString = e.getValue();
                longStringVarName = e.getKey();
                continue;
            }
            writer.append(types.get(e.getKey()));
            writer.append(" ");
            writer.append(e.getKey().replace(' ', '-'));
            writer.append(" ");
            writer.append(getStringRep(e.getValue(), true));
            writer.append("\n");
        }
        if(longString != null) {
            writer.append("LongString ");
            writer.append(longStringVarName);
            writer.append("\n");
            writer.append(longString.toString());
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

            throw new IllegalArgumentException("that variable doesn't exist: " + name);
        }
        Object val = stats.get(name);
        // return a copy if returning an array.
        if(val instanceof int[] || val instanceof String[]) {
            Object copy;
            if(val instanceof int[]) {
                int len = ((int[])val).length;
                copy = new int[len];
                for(int i = 0; i < len; i++) {
                    ((int[])copy)[i] = ((int[])val)[i];
                }
            } else {
                int len = ((String[])val).length;
                copy = new String[len];
                for(int i = 0; i < len; i++) {
                    ((String[])copy)[i] = ((String[])val)[i];
                }
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
        stats.put(name, value);
        if(value instanceof Integer) {
            types.put(name, "int");
        } else if(value instanceof String) {
            types.put(name, "String");
        } else if(value instanceof int[]) {
            types.put(name, "int[]");
        } else if (value instanceof String[]) {
            types.put(name, "String[]");
        } else if (value instanceof StringBuilder) {
            types.put(name, "LongString");
        } else {
            types.put(name, "double");
        }
    }

    public void removeVariable(String name) {
        if(!types.containsKey(name)) {
            throw new IllegalArgumentException("That variable doesn't exist!");
        }
        types.remove(name);
        stats.remove(name);
    }

    // check if provided object is one of the types we can parse.
    private boolean isValidType(Object o) {
        return o instanceof Integer ||
               o instanceof String ||
               o instanceof int[] ||
               o instanceof String[] ||
               o instanceof StringBuilder ||
               o instanceof Double;
    }

    // string rep of a value. 
    private String getStringRep(Object o, boolean replaceSpaces) {
        if(o instanceof Object[]) {
            Object[] arr = (Object[])o;
            StringBuilder s = new StringBuilder();
            for(int i = 0; i < arr.length; i++) {
                if(arr[i] instanceof String) {
                    if(replaceSpaces) {
                        s.append(((String)arr[i]).replace(' ', '-'));
                    } else {
                        s.append((String)arr[i]);
                    }
                } else {
                    s.append(((Integer)arr[i]) + "");
                }
                if(i != arr.length - 1) {
                    s.append(" ");
                }
            }
            return s.toString();
        } else {
            if(replaceSpaces && !(o instanceof StringBuilder)) {
                return o.toString().replace(' ', '-');
            } else {
                return o.toString();
            }
        }
    }

    // string rep of all the stats.
    public String toString() {
        StringBuilder s = new StringBuilder();        
        for(String prop : properties) {
            s.append(prop);
            s.append("\n");
        }
        for(Map.Entry<String, Object> e : stats.entrySet()) {
            s.append(e.getKey());
            s.append(": ");
            s.append(getStringRep(e.getValue(), false));
            s.append("\n");
        }
        return s.toString();
    }

    public static class ReadOnlyStats {
        private Stats stats;
        public ReadOnlyStats(Stats stats) {
            this.stats = stats;
        }
        public boolean hasVariable(String stat) {
            return stats.hasVariable(stat);
        }
        public Object get(String stat) {
            return stats.get(stat);
        }
        public boolean hasProperty(String property) {
            return stats.hasProperty(property);
        }
        public String toString() {
            return stats.toString();
        }
        public void saveTo(String file) throws FileNotFoundException {
            stats.saveTo(file);
        }
    }
}
