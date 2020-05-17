package server.objects.instantiables;

import server.objects.Entity;
import server.objects.Instantiable;
import server.objects.Stats;

public class Item implements Instantiable<Item> {
    private Stats stats;

    public Item() {}

    public Item(Stats.ReadOnlyStats stats) {
        this.stats = stats.clone();
    }

    public Stats.ReadOnlyStats getStats() {
        return new Stats.ReadOnlyStats(stats);
    }

    @Override
    public Item create(Entity e) {
        return new Item(e.getStats());
    }
}
