package server.objects;

public interface ValueType<T> {
    public int getID();

    public Stats.ReadOnlyStats getStats();

    public T create(int ID, Stats.ReadOnlyStats stats);
}