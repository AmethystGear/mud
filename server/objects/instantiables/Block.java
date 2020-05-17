package server.objects.instantiables;

import server.objects.Entity;
import server.objects.Instantiable;
import server.objects.Stats;

public class Block implements Instantiable<Block> {
    private int ID;
    private Stats.ReadOnlyStats stats;

    public Block() {
    }

    public Block(int ID, Stats.ReadOnlyStats stats) {
        this.ID = ID;
        this.stats = stats;
    }

    public int getID() {
        return ID;
    }

    public Stats.ReadOnlyStats getStats() {
        return stats;
    }

    @Override
    public Block create(Entity e) {
        return new Block(e.ID(), e.getStats());
    }
}
