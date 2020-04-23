package server.objects;
import java.io.File;
import java.io.PrintWriter;
import java.io.IOException;
import java.util.HashMap;
import java.util.Scanner;
import java.util.Collection;
import java.util.Collections;

public class Value {
    public static <T extends ValueType<T>> ValueSet<T> getValuesFromFile(String file, String valueEndString, T value) {
        try {
            Scanner s = new Scanner(new File(file));
            ValueSet<T> valueSet = new ValueSet<T>();
            int ID = 0;
            while(s.hasNextLine()) {
                if(s.nextLine().contains(valueEndString)) {
                    return valueSet;
                }
                Stats stats = new Stats(s);
                valueSet.add(value.create(ID, new Stats.ReadOnlyStats(stats)));
                ID++;
            }
            return valueSet;           
        } catch (IOException e) {
            throw new IllegalArgumentException();
        }
    }

    public static <T extends ValueType<T>> ValueSet<T> getValuesFromFile(String file, T value) {
        return getValuesFromFile(file, "\n", value); // this works because Scanner.nextLine() will never contain a \n character.
    }

    public static <T extends ValueType<T>> void saveTo(ValueSet<T> v, String endString, PrintWriter p) {
        p.write("\n");
        for(T value : v.values()) {
            p.write("\n");
            value.getStats().saveTo(p);
            p.write("\n");
        }
        p.write(endString);
        p.write("\n");
    }

    public static class ValueSet<U extends ValueType<U>> {
        private HashMap<String, U> nameToValue;
        private HashMap<Integer, U> IDtoValue;
    
        public ValueSet() {
            nameToValue = new HashMap<String, U>();
            IDtoValue = new HashMap<Integer, U>();
        }
    
        private void add(U b) {
            nameToValue.put((String)b.getStats().get("name"), b);
            IDtoValue.put(b.getID(), b);
        }
    
        public U get(int ID) {
            if(!IDtoValue.containsKey(ID)) {
                throw new IllegalArgumentException("that value doesn't exist: " + ID);
            }
            return IDtoValue.get(ID);
        }
    
        public U get(String name) {
            if(!nameToValue.containsKey(name)) {
                throw new IllegalArgumentException("that value doesn't exist: " + name);
            }
            return nameToValue.get(name);
        }

        public Collection<U> values() {
            return Collections.unmodifiableCollection(IDtoValue.values());
        }

        public int size() {
            return IDtoValue.size();
        }
    }
}