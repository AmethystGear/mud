package server.utils;

public class MultiDimensionalFloatArray {
    private float[] backing;
    private int[] dimensions;

    public MultiDimensionalFloatArray(int... dimensions) {
        this.dimensions = new int[dimensions.length];
        int len = 1;
        for(int i = 0; i < dimensions.length; i++) {
            this.dimensions[i] = dimensions[i];
            len *= dimensions[i];
        }
        backing = new float[len];
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

    public float get(int index) {
        return backing[index];
    }

    public void set(float val, int index) {
        backing[index] = val;
    }

    public float get(int... dim) {
        return backing[getIndex(dim)];
    }

    public void set(float val, int... dim) {
        backing[getIndex(dim)] = val;
    }
}
