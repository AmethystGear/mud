package server.objects;

public class Int {
    int i;
    public Int(int i) {
        this.i = i;
    }

    public int get() {
        return i;
    }

    public void set(int i) {
        this.i = i;
    }

    public static class ReadOnlyInt {
        Int i;
        public ReadOnlyInt(Int i) {
            this.i = i;
        }

        public int get() {
            return i.get();
        }
    }
}