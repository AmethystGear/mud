package server.utils;

public class MultiDimensionalIntArray {
    private int[] backing;
    private int[] dimensions;

    public MultiDimensionalIntArray(int... dimensions) {
        this.dimensions = new int[dimensions.length];
        int len = 1;
        for(int i = 0; i < dimensions.length; i++) {
            this.dimensions[i] = dimensions[i];
            len *= dimensions[i];
        }
        backing = new int[len];
    }

    private int getIndex(int... dim) {
        if (dim.length != dimensions.length) {
            throw new IllegalArgumentException();
        }
        int index = 0;
        int factor = 1;
        for(int i = 0; i < dim.length; i++) {
            index += factor * dim[i];
            factor *= dimensions[i];
        }
        return index;
    }

    public int get(int index) {
        return backing[index];
    }

    public void set(int val, int index) {
        backing[index] = val;
    }

    public int get(int... dim) {
        return backing[getIndex(dim)];
    }

    public void set(int val, int... dim) {
        backing[getIndex(dim)] = val;
    }
}
