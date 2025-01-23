package co.realfit.example;

import android.app.NativeActivity;
import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Context;
import android.os.Bundle;
import android.util.Log;
import android.view.inputmethod.InputMethodManager;

public class MainActivity extends NativeActivity {

    static {
        System.loadLibrary("example");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
    }

    private void showKeyboard() {
        Log.d("MainActivity", "showKeyboard instance method called");
        InputMethodManager inputManager = getSystemService(InputMethodManager.class);
        inputManager.showSoftInput(getWindow().getDecorView(), InputMethodManager.SHOW_IMPLICIT);
    }

    private void hideKeyboard() {
        Log.d("MainActivity", "hideKeyboard instance method called");
        InputMethodManager inputManager = getSystemService(InputMethodManager.class);
        inputManager.hideSoftInputFromWindow(getWindow().getDecorView().getWindowToken(), 0);
    }

    private String readClipboard() {
        ClipboardManager clipboardManager = (ClipboardManager) getApplicationContext().getSystemService(Context.CLIPBOARD_SERVICE);
        ClipData data = clipboardManager.getPrimaryClip();
        if (data == null) {
            Log.d("MainActivity", "ClipData in readClipboard is null");
            return "";
        }
        ClipData.Item item = data.getItemAt(0);
        if (item == null) {
            Log.d("MainActivity", "Item in readClipboard is null");
            return "";
        }
        return item.coerceToText(this).toString();
    }

    private void writeClipboard(String value) {
        ClipboardManager clipboardManager = (ClipboardManager) getApplicationContext().getSystemService(Context.CLIPBOARD_SERVICE);
        ClipData data = ClipData.newPlainText("MainActivity text", value);
        clipboardManager.setPrimaryClip(data);
    }
}
