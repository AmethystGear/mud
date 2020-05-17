package server.objects;

public interface Instantiable<T> {
    public T create(Entity e);
}
