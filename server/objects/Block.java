public class Block implements ValueType<Block>{
    private int ID;
    private ReadOnlyStats stats;

    public Block(){}

    public Block(int ID, ReadOnlyStats stats) {
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
    public Block create(int ID, ReadOnlyStats stats) {
        return new Block(ID, stats);
    }
}