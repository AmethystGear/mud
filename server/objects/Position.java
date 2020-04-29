package server.objects;

public class Position {
    int x;
    int y;

    public Position(int x, int y) {
        this.x = x;
        this.y = y;
    }

    public void moveTo(int x, int y) {
        this.x = x;
        this.y = y;
    }

    public int x() {
        return x;
    }

    public int y() {
        return y;
    }

    public String toString() {
        return "position: " + x + ", " + y;
    }

    public static class ReadOnlyPosition {
        Position posn;
        public ReadOnlyPosition(Position posn) {
            this.posn = posn;
        }

        public int x() {
            return posn.x();
        }

        public int y() {
            return posn.y();
        }

        public String toString() {
            return posn.toString();
        }
    }
}