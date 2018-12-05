#[cfg(test)]
mod tests {
    use microbe::constants::*;
    use microbe::*;

    fn example_post() -> Post {
        Post {
            content: "Hello World, how are you today?".to_string(),
            timestamp: DateTime::from_utc(
                chrono::naive::NaiveDateTime::from_timestamp(0, 0),
                chrono::offset::Utc,
            ),
        }
    }

    #[test]
    fn post_to_rss() {
        assert_eq!(
            example_post().as_rss(),
            format!(include_str!("templates/post.rss")
        , link = constants::HOSTNAME));
    }

    #[test]
    fn post_to_atom() {
        assert_eq!(
            example_post().as_atom(),
            include_str!("templates/post.rss")
        );
    }

    #[test]
    fn post_to_json() {
        assert_eq!(
            example_post().as_json(),
            "{
    \"id\": \"86675203-8e40-5254-94ea-8c8f6f255bf1\",
    \"content_text\": \"Hello World, how are you today?\",
    \"url\": \"/0_hello-world-how-are-you\",
    \"date_published\": \"1970-01-01T00:00:00+00:00\"
}"
        );
    }

    #[test]
    fn _create_rss_from_posts() {
        assert_eq!(
            create_rss_from_posts(&vec![example_post()]),
            format!("<?xml version=\"1.0\" encoding=\"UTF-8\" ?>
<rss version=\"2.0\">
    <channel>
        <title>{user}</title>
        <description>microbe feed by {user}</description>
        <link>{link}</link>
        <lastBuildDate>{now}</lastBuildDate>
        <item>
    <title>1970-01-01T00:00:00+00:00</title>
    <description>Hello World, how are you today?</description>
    <link>{link}/0_hello-world-how-are-you</link>
    <guid isPermaLink=\"false\">86675203-8e40-5254-94ea-8c8f6f255bf1</guid>
</item>
    </channel>
</rss>",
user = constants::USERNAME,
link = constants::HOSTNAME,
now = Utc::now().to_rfc2822()))
    }

    #[test]
    fn _create_atom_from_posts() {
        assert_eq!(
            create_atom_from_posts(&vec![example_post()]),
            format!("<?xml version=\"1.0\" encoding=\"UTF-8\" ?>
<feed xmlns=\"http://www.w3.org/2005/Atom\">
    <title>{user}</title>
    <subtitle>microbe feed by {user}</subtitle>
    <link href=\"{host}/feed.atom\" rel=\"self\" />
    <link href=\"{host}\" />
    <id>{id}</id>
    <updated>{now}</updated>

    <entry>
    <title>1970-01-01T00:00:00+00:00</title>
    <link type=\"text/html\" href=\"/0_hello-world-how-are-you\" />
    <id>urn:uuid:86675203-8e40-5254-94ea-8c8f6f255bf1</id>
    <updated>1970-01-01T00:00:00+00:00</updated>
    <author><name>microbe user</name></author>
    <content type=\"text\">
        Hello World, how are you today?
    </content>
</entry>
</feed>",
user = constants::USERNAME,
host = constants::HOSTNAME,
now = Utc.timestamp(0, 0).to_rfc2822(),
id = uuid!(constants::USERNAME).urn()))
    }

    #[test]
    fn _create_json_from_posts() {
        assert_eq!(
            create_json_from_posts(&vec![example_post()]),
            format!("{{
    \"version\": \"https://jsonfeed.org/version/1\",
    \"title\": \"@{user}\",
    \"home_page_url\": \"{host}\",
    \"feed_url\": \"{host}/feed.json\",
    \"items\": [
        {{
    \"id\": \"86675203-8e40-5254-94ea-8c8f6f255bf1\",
    \"content_text\": \"Hello World, how are you today?\",
    \"url\": \"/0_hello-world-how-are-you\",
    \"date_published\": \"1970-01-01T00:00:00+00:00\"
}}
    ]
}}",
user = constants::USERNAME,
host = constants::HOSTNAME));
}
}
