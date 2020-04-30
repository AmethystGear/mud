package server.objects;

public class Block implements ValueType<Block> {
    private int ID;
    private Stats.ReadOnlyStats stats;

    public Block() {
    }

    public Block(int ID, Stats.ReadOnlyStats stats) {
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
    public Block create(int ID, Stats.ReadOnlyStats stats) {
        return new Block(ID, stats);
    }

    public int display() {
        return (Integer)stats.get("display");
    }
}