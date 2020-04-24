package server.objects;

public class Item implements ValueType<Item> {
    private int ID;
    private Stats.ReadOnlyStats stats;

    public Item() {
    }

    public Item(int ID, Stats.ReadOnlyStats stats) {
        this.ID = ID;
        this.stats = stats;
    }

    @Override
    public int getID() {
        return ID;
    }

    @Override
    public Stats.ReadOnlyStats getStats() {
        return stats;
    }

    @Override
    public Item create(int ID, Stats.ReadOnlyStats stats) {
        return new Item(ID, stats);
    }
}