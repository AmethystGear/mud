public class Item implements ValueType<Item>{
    private int ID;
    private ReadOnlyStats stats;

    public Item(){}

    public Item(int ID, ReadOnlyStats stats) {
        this.ID = ID;
        this.stats = stats;
    }

    @Override
    public int getID() {
        return ID;
    }

    @Override
    public ReadOnlyStats getStats() {
        return stats;
    }

    @Override
    public Item create(int ID, ReadOnlyStats stats) {
        return new Item(ID, stats);
    }
}