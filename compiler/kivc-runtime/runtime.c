#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

// Text type with Copy-on-Write semantics (RFC 0002)
typedef struct {
    char* data;        // UTF-8 string data
    size_t length;     // byte length
    size_t capacity;   // allocated capacity
    size_t* ref_count; // reference counter for CoW
} Text;

// Create Text from C string
Text* kiv_text_from_cstr(const char* str) {
    size_t len = strlen(str);
    Text* text = (Text*)malloc(sizeof(Text));
    if (!text) {
        fprintf(stderr, "Out of memory\n");
        exit(1);
    }
    
    text->length = len;
    text->capacity = len + 1;
    text->data = (char*)malloc(text->capacity);
    text->ref_count = (size_t*)malloc(sizeof(size_t));
    
    if (!text->data || !text->ref_count) {
        fprintf(stderr, "Out of memory\n");
        exit(1);
    }
    
    memcpy(text->data, str, len + 1);
    *text->ref_count = 1;
    
    return text;
}

// Clone Text (increment reference count for CoW)
Text* kiv_text_clone(Text* text) {
    if (!text) return NULL;
    
    (*text->ref_count)++;
    return text;
}

// Drop Text (decrement reference count, free if zero)
void kiv_text_drop(Text* text) {
    if (!text) return;
    
    (*text->ref_count)--;
    if (*text->ref_count == 0) {
        free(text->data);
        free(text->ref_count);
        free(text);
    }
}

// Concatenate two Text values (creates new Text)
Text* kiv_text_concat(Text* a, Text* b) {
    if (!a || !b) return NULL;
    
    size_t new_len = a->length + b->length;
    Text* result = (Text*)malloc(sizeof(Text));
    if (!result) {
        fprintf(stderr, "Out of memory\n");
        exit(1);
    }
    
    result->length = new_len;
    result->capacity = new_len + 1;
    result->data = (char*)malloc(result->capacity);
    result->ref_count = (size_t*)malloc(sizeof(size_t));
    
    if (!result->data || !result->ref_count) {
        fprintf(stderr, "Out of memory\n");
        exit(1);
    }
    
    memcpy(result->data, a->data, a->length);
    memcpy(result->data + a->length, b->data, b->length);
    result->data[new_len] = '\0';
    *result->ref_count = 1;
    
    return result;
}

// Compare two Text values for equality
bool kiv_text_equals(Text* a, Text* b) {
    if (!a || !b) return false;
    if (a == b) return true;
    if (a->length != b->length) return false;
    return memcmp(a->data, b->data, a->length) == 0;
}

// Built-in print functions (no newline for multi-arg support)
void kiv_print_text(Text* text) {
    if (text && text->data) {
        printf("%s", text->data);
    } else {
        printf("(null)");
    }
}

void kiv_print_int(int64_t value) {
    printf("%lld", (long long)value);
}

void kiv_print_float(double value) {
    printf("%g", value);
}

void kiv_print_bool(bool value) {
    printf("%s", value ? "true" : "false");
}

// Print newline (called after all print arguments)
void kiv_print_newline(void) {
    printf("\n");
}

// Conversion functions (stubs)
Text* kiv_int_to_text(int64_t value) {
    (void)value;
    return NULL;
}

Text* kiv_float_to_text(double value) {
    (void)value;
    return NULL;
}

Text* kiv_bool_to_text(bool value) {
    (void)value;
    return NULL;
}

// Panic function
void kiv_panic(Text* message) {
    if (message && message->data) {
        fprintf(stderr, "panic: %s\n", message->data);
    } else {
        fprintf(stderr, "panic\n");
    }
    exit(1);
}

