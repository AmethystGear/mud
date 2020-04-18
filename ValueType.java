public interface ValueType<T> {
    public int getID();
    public ReadOnlyStats getStats();
    public T create(int ID, ReadOnlyStats stats);
}