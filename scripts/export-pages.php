#!/usr/bin/env php
<?php
/**
 * Export PAGE_INFO from URLParse.php as JSON.
 *
 * Usage: php scripts/export-pages.php > data/pages.json
 */

// Load the URLParse class
require_once __DIR__ . '/../../defuse.ca/src/libs/URLParse.php';

// Map numeric keys to readable names
$key_names = [
    P_FILE => 'file',
    P_TITL => 'title',
    P_METD => 'description',
    P_METK => 'keywords',
    P_RDIR => 'redirect',
];

// Get the PAGE_INFO array and convert keys
$pages = [];
foreach (URLParse::$PAGE_INFO as $slug => $info) {
    $page = [];
    foreach ($info as $key => $value) {
        $name = $key_names[$key] ?? $key;
        $page[$name] = $value;
    }
    $pages[$slug] = $page;
}

// Output as pretty-printed JSON
echo json_encode($pages, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES) . "\n";
