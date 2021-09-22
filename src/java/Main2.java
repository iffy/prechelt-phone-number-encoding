import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.FileReader;
import java.io.IOException;
import java.io.OutputStreamWriter;
import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.function.BiConsumer;
import java.util.stream.Stream;

import static java.nio.charset.StandardCharsets.US_ASCII;

/**
 * This is a Java port of the Norvig's Common Lisp program from http://norvig.com/java-lisp.html.
 * <p>
 * The only relevant deviations from the original are:
 * <ul>
 *     <li>
 *     The words for a solution are accumulated using {@code cons} in CL, but using {@code List.add} in Java.
 *     This means that when a solution is found, the Java program doesn't need to reverse the words before printing
 *     them. Also, the Java program checks the <b>last</b> word when deciding if the previous word in a solution was
 *     a digit, while the CL program checks the <b>first</b> word.
 *     </li>
 *     <li>
 *     The {@code printTranslations} method in Java actually does not directly print the solutions,
 *     but takes a {@link BiConsumer} as an argument, which is called with each (number, solution) found.
 *     This was done to make the code more easily testable.
 *     </li>
 * </ul>
 *
 * @author Renato Athaydes
 */
final class Main2 {
    public static void main( String[] args ) throws IOException, InterruptedException {
        var words = new BufferedReader( new FileReader(
                args.length > 0 ? args[ 0 ] : "tests/words.txt", US_ASCII ) ).lines();


        var printer = new BufferedWriter( new OutputStreamWriter( System.out, US_ASCII ) );

        try ( printer ) {
            var encoder = new PhoneNumberEncoder2( words, printer );
            new BufferedReader( new FileReader(
                    args.length > 1 ? args[1] : "tests/numbers.txt", US_ASCII )
            ).lines().forEach( encoder::encode );
        }
    }
}

final class PhoneNumberEncoder2 {
    private final Map<ByteBuffer, List<String>> dict;
    private final BufferedWriter printer;

    PhoneNumberEncoder2(Stream<String> words, BufferedWriter printer) {
        this.printer = printer;
        dict = loadDictionary( words );
    }

    void encode( String phone ) {
        printTranslations( phone, removeIfNotLetterOrDigit( phone.toCharArray() ), 0, new ArrayList<>() );
    }

    private byte[] removeIfNotLetterOrDigit( char[] chars ) {
        var finalLength = 0;
        for ( char c : chars ) {
            if ( Character.isLetterOrDigit( c ) ) finalLength++;
        }
        byte[] result = new byte[ finalLength ];
        var i = 0;
        for ( char c : chars ) {
            if ( Character.isLetterOrDigit( c ) ) {
                result[ i ] = ( byte ) c;
                i++;
            }
        }
        return result;
    }

    private void printTranslations( String num, byte[] digits, int start, List<String> words ) {
        if ( start >= digits.length ) {
            printSolution( num, words );
            return;
        }
        var foundWord = false;
        var bytes = ByteBuffer.allocate( digits.length - start );
        toReadState( bytes );
        for ( int i = start; i < digits.length; i++ ) {
            bytes = toWriteState( bytes ).put( nthDigit( digits, i ) );
            List<String> foundWords = dict.get( toReadState( bytes ) );
            if ( foundWords != null ) {
                foundWord = true;
                for ( String word : foundWords ) {
                    words.add( word );
                    printTranslations( num, digits, i + 1, words );
                    words.remove( words.size() - 1 );
                }
            }
        }
        if ( !foundWord && !isLastItemDigit( words ) ) {
            words.add( Integer.toString( nthDigit( digits, start ) ) );
            printTranslations( num, digits, start + 1, words );
            words.remove( words.size() - 1 );
        }
    }

    private void printSolution( String num, List<String> words ) {
        // writing it out by invoking println several times like Rust does is counter-productive
        try {
            printer.write( num + ": " + String.join( " ", words ) );
            printer.write( '\n' );
        } catch ( IOException e ) {
            throw new RuntimeException( e );
        }
    }

    private static Map<ByteBuffer, List<String>> loadDictionary( Stream<String> words ) {
        var table = new HashMap<ByteBuffer, List<String>>( 100 );
        words.forEach( word -> table.computeIfAbsent( wordToNumber( word ),
                ( ignore ) -> new ArrayList<>() ).add( word ) );
        return table;
    }

    private boolean isLastItemDigit( List<String> words ) {
        if ( words.isEmpty() ) return false;
        var lastWord = words.get( words.size() - 1 );
        return lastWord.length() == 1 && Character.isDigit( lastWord.chars().sum() );
    }

    private static ByteBuffer wordToNumber( String word ) {
        ByteBuffer bytes = ByteBuffer.allocate( word.length() );
        for ( char c : word.toCharArray() ) {
            if ( Character.isLetter( c ) ) {
                bytes.put( charToDigit( c ) );
            }
        }
        return toReadState( bytes );
    }

    private static ByteBuffer toReadState( ByteBuffer bytes ) {
        bytes.limit( bytes.position() );
        bytes.position( 0 );
        return bytes;
    }

    private static ByteBuffer toWriteState( ByteBuffer bytes ) {
        var pos = bytes.limit();
        bytes.limit( bytes.capacity() );
        bytes.position( pos );
        return bytes;
    }

    private static byte nthDigit( byte[] digits, int n ) {
        return ( byte ) ( digits[ n ] - ( byte ) '0' );
    }

    static byte charToDigit( char c ) {
        return switch ( Character.toLowerCase( c ) ) {
            case 'e' -> ( byte ) 0;
            case 'j', 'n', 'q' -> ( byte ) 1;
            case 'r', 'w', 'x' -> ( byte ) 2;
            case 'd', 's', 'y' -> ( byte ) 3;
            case 'f', 't' -> ( byte ) 4;
            case 'a', 'm' -> ( byte ) 5;
            case 'c', 'i', 'v' -> ( byte ) 6;
            case 'b', 'k', 'u' -> ( byte ) 7;
            case 'l', 'o', 'p' -> ( byte ) 8;
            case 'g', 'h', 'z' -> ( byte ) 9;
            default -> throw new RuntimeException( "Invalid char: " + c );
        };
    }

}
