import java.io.*;
import java.util.*;

public class Value {
    public static <T extends ValueType<T>> ValueSet<T> getValueFromFile(String file, T value) {
        try {
            Scanner s = new Scanner(new File(file));
            ValueSet<T> valueSet = new ValueSet<T>();
            int ID = 0;
            while(s.hasNextLine()) {
                Stats stats = new Stats(s);
                valueSet.add(value.create(ID, new ReadOnlyStats(stats)));
                ID++;
            }
            return valueSet;           
        } catch (IOException e) {
            throw new IllegalArgumentException();
        }
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
            if(!blockIDtoValue.containsKey(ID)) {
                throw new IllegalArgumentException("that value doesn't exist: " + ID);
            }
            return blockIDtoValue.get(ID);
        }
    
        public U get(String name) {
            if(!nameToValue.containsKey(name)) {
                throw new IllegalArgumentException("that value doesn't exist: " + name);
            }
            return nameToValue.get(name);
        }
    }
}