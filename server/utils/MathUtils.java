package server.utils;

public class MathUtils {
    public static int manhattan(int x1, int y1, int x2, int y2) {
        return abs(x1 - x2) + abs(x2 - y2);
    }

    public static int euclideanSqr(int x1, int y1, int x2, int y2) {
        return (x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2);
    }

    public static int sign(int a) {
        if (a > 0) {
            return 1;
        } else if (a == 0) {
            return 0;
        } else {
            return -1;
        }
    }

    public static int abs(int a) {
        return a < 0 ? -a : a;
    }

    public static int min(int a, int b) {
        return a < b ? a : b;
    }

    public static int max(int a, int b) {
        return a > b ? a : b;
    }

    public static int bound(int a, int min, int max) {
        return max(min, min(max, a));
    }
}