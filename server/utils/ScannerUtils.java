package server.utils;

import java.util.*;

public class ScannerUtils {
    public static String[] getRemainingInputAsStringArray(Scanner s) {
        ArrayList<String> a = new ArrayList<>();
        while(s.hasNext()) {
            a.add(s.next().replace('_', ' ').replace('-', ' '));
        }
        String [] arr = new String[a.size()];
        for(int i = 0; i < arr.length; i++) {
            arr[i] = a.get(i);
        }
        return arr;
    }

    public static String getRemainingInputAsString(Scanner s) {
        String [] input = getRemainingInputAsStringArray(s);
        if(input.length > 0) {
            StringBuilder concat = new StringBuilder(input[0]);
            for(int i = 1; i < input.length; i++) {
                concat.append(" ");
                concat.append(input[i]);
            }
            return concat.toString();
        } else {
            return "";
        }
    }

    public static StringBuilder getInputTillEnd(Scanner s) {
        StringBuilder str = new StringBuilder();
        while(s.hasNextLine()) {
            String line = s.nextLine();
            if(line.contains("/end/")) {
                return str;
            }
            str.append(line);
            str.append("\n");
        }
        return str;
    }
}