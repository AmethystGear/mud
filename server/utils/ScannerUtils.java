package server.utils;

import java.util.*;

// contains static methods used to get values from scanners.
public class ScannerUtils {
    
    /**
     * return the tokens remaining in s as a String[], replacing all '_' and '-' with spaces.
     * @param s scanner that we will convert to String []
     * @return String[] containing the remaining input, where each token is an element in the array.
     */
    public static String[] getRemainingInputAsStringArray(Scanner s) {
        ArrayList<String> a = new ArrayList<>();
        while (s.hasNext()) {
            a.add(s.next().replace('_', ' ').replace('-', ' '));
        }
        String[] arr = new String[a.size()];
        for (int i = 0; i < arr.length; i++) {
            arr[i] = a.get(i);
        }
        return arr;
    }


    /**
     * return String containing all the remaining tokens, with a space between each token.
     * all '_' and '-' in the original input are replaced with spaces.
     * @param s scanner that we will convert to String
     * @return String containing the rest of the input.
     */
    public static String getRemainingInputAsString(Scanner s) {
        String[] input = getRemainingInputAsStringArray(s);
        if (input.length > 0) {
            StringBuilder concat = new StringBuilder(input[0]);
            for (int i = 1; i < input.length; i++) {
                concat.append(" ");
                concat.append(input[i]);
            }
            return concat.toString();
        } else {
            return "";
        }
    }

    /**
     * return StringBuilder containing scanner input till "/end/"
     * @param s scanner that we will convert to StringBuilder
     * @return StringBuilder containing each line of input up till a line that contains the string "/end/"
     */
    public static StringBuilder getInputTillEnd(Scanner s) {
        StringBuilder str = new StringBuilder();
        while (s.hasNextLine()) {
            String line = s.nextLine();
            if (line.contains("/end/")) {
                return str;
            }
            str.append(line);
            str.append("\n");
        }
        return str;
    }
}