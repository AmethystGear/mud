package server.objects;
import java.io.PrintWriter;
import java.util.Collection;
import java.util.Collections;
import java.util.HashMap;
import java.util.Scanner;

import server.objects.Stats.ReadOnlyStats;

public class Entity {
    private int ID;
    private ReadOnlyStats stats;

    public Entity(int ID, Stats stats) {
        this.ID = ID;
        this.stats = new ReadOnlyStats(stats);
    }

    public Entity(int ID, ReadOnlyStats stats) {
        this.ID = ID;
        this.stats = stats;
    }

    public ReadOnlyStats getStats() {
        return stats;
    }

    public int ID() {
        return ID;
    }

    public static class EntitySet {
        private static int ID = 0;

        private HashMap<String, Entity> nameToValue;
        private HashMap<Integer, Entity> IDtoValue;

        public EntitySet() {
            nameToValue = new HashMap<String, Entity>();
            IDtoValue = new HashMap<Integer, Entity>();
        }

        public void add(Scanner s) {
            add(s, new Stats());
        }

        public void add(Scanner s, Stats appendedStats) {
            while (s.hasNextLine()) {
                String nextLine = "";
                while (nextLine.equals("")) {
                    nextLine = s.nextLine();
                }
                Stats stats = new Stats(s);
                stats.add(appendedStats);
                add(new Entity(ID, stats));
                ID++;
            }
        }

        private void add(Entity b) {
            System.out.println(b.getStats());
            nameToValue.put((String) b.getStats().get("name"), b);
            IDtoValue.put(b.ID(), b);
        }

        public Entity get(int ID) {
            if (!IDtoValue.containsKey(ID)) {
                throw new IllegalArgumentException("that value doesn't exist: " + ID);
            }
            return IDtoValue.get(ID);
        }

        public Entity get(String name) {
            if (!nameToValue.containsKey(name)) {
                throw new IllegalArgumentException("that value doesn't exist: " + name);
            }
            return nameToValue.get(name);
        }

        public Collection<Entity> values() {
            return Collections.unmodifiableCollection(IDtoValue.values());
        }

        public int size() {
            return IDtoValue.size();
        }

        public void saveTo(PrintWriter p) {
            p.write("\n");
            for (Entity value : IDtoValue.values()) {
                p.write("\n");
                value.getStats().saveTo(p);
                p.write("\n");
            }
            p.write("\n");
        }
    }
}
