extern crate chrono;
#[macro_use]
extern crate slugify;
extern crate uuid;

use chrono::prelude::*;
use slugify::slugify;
use uuid::Uuid;
use std::io::BufRead;
use std::str::FromStr;

// Constant variables.
static USERNAME: &str = "microbeuser";
static POSTS_FILE: &str = "microbes";
static HOSTNAME: &str = "https://example.edu";

macro_rules! slug {
    ($c:expr, $t:expr) => (
        format!("{}_{}", $t.timestamp(), slugify!(&$c, max_length = 24))
    )
}

macro_rules! uuid {
    ($t:expr) => (
        Uuid::new_v5(&uuid::NAMESPACE_URL, &$t.to_string())
    )
}

#[derive(Debug, Clone)]
struct Post {
    content: String,
    timestamp: DateTime<chrono::offset::Utc>,
}

impl Post {
    fn as_rss(&self) -> String {
        format!(
            "<item>
    <title>{time}</title>
    <description>{content}</description>
    <link>{host}/{slug}</link>
    <guid isPermaLink=\"false\">{id}</guid>
</item>",
            id = uuid!(self.timestamp),
            time = self.timestamp.to_rfc3339(),
            slug = slug!(self.content, self.timestamp),
            content = self.content,
            host = HOSTNAME
        )
    }

    fn as_atom(&self) -> String {
        format!(
            "<entry>
    <title>{time}</title>
    <link type=\"text/html\" href=\"/{slug}\" />
    <id>{id}</id>
    <updated>{time}</updated>
    <author><name>microbe user</name></author>
    <content type=\"text\">
        {content}
    </content>
</entry>",
            id = uuid!(self.timestamp).urn(),
            time = self.timestamp.to_rfc3339(),
            slug = slug!(self.content, self.timestamp),
            content = self.content
        )
    }

    fn as_json(&self) -> String {
        format!(
            "{{
    \"id\": \"{id}\",
    \"content_text\": \"{content}\",
    \"url\": \"/{slug}\",
    \"date_published\": \"{time}\"
}}",
            id = uuid!(self.timestamp),
            content = self.content,
            slug = slug!(self.content, self.timestamp),
            time = self.timestamp.to_rfc3339()
        )
    }
}

fn get_posts(filename: &str) -> Vec<Post> {
    macro_rules! remove_last_space {
        ($t:expr) => ({
            let mut post = $t.unwrap();
            let mut content = post.content;
            content.pop();
            post.content = content;
            $t = Some(post);
        })
    }

    let mut ret: Vec<Post> = Vec::new();
    let mut post_buffer: Option<Post> = None;

    for _line in std::io::BufReader::new(&std::fs::File::open(filename).unwrap()).lines() {
        let line = _line.unwrap();

        if line.starts_with("@@ ") {
            if post_buffer.clone().is_some() {
                remove_last_space!(post_buffer);
                ret.push(post_buffer.unwrap());
            }

            post_buffer = Some(Post {
                timestamp: DateTime::<Utc>::from_str(
                    line.split(" ").nth(1).unwrap_or("1970-01-01T00:00+00:00"),
                ).unwrap(),
                content: "".to_string(),
            });
        } else {
            if post_buffer.is_none() {
                continue;
            }

            let mut post = post_buffer.unwrap();
            let mut content = post.content.to_string();
            content.push_str(&line);
            content.push(' ');
            post.content = content;
            post_buffer = Some(post);
        }
    }

    if post_buffer.is_some() {
        remove_last_space!(post_buffer);
        ret.push(post_buffer.unwrap());
    }

    return ret;
}

fn create_rss_from_posts(posts: &Vec<Post>) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>
<rss version=\"2.0\">
    <channel>
        <title>{user}</title>
        <description>microbe feed by {user}</description>
        <link>{link}</link>
        <lastBuildDate>{now}</lastBuildDate>
        {feeds}
    </channel>
</rss>",
        user = USERNAME,
        link = HOSTNAME,
        now = Utc::now().to_rfc2822(),
        feeds = {
            let mut ret = "".to_string();

            for post in posts {
                ret.push_str(&post.as_rss());
            }

            ret
        }
    )
}

fn create_atom_from_posts(posts: &Vec<Post>) -> String {
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\" ?>
<feed xmlns=\"http://www.w3.org/2005/Atom\">
    <title>{user}</title>
    <subtitle>microbe feed by {user}</subtitle>
    <link href=\"{host}/feed.atom\" rel=\"self\" />
    <link href=\"{host}\" />
    <id>{id}</id>
    <updated>{now}</updated>

    {feeds}
</feed>",
user = USERNAME,
host = HOSTNAME,
now = {
    // TODO refactor this code
    #[cfg(test)]
    {
        Utc.timestamp(0, 0)
    }
    #[cfg(not(test))]
    {
        Utc::now()
    }
}.to_rfc2822(),
id = uuid!(USERNAME).urn(),
feeds = {
    let mut ret = "".to_string();

    for post in posts {
        ret.push_str(&post.as_atom());
    }

    ret
})
}

fn create_json_from_posts(posts: &Vec<Post>) -> String {
    format!("{{
    \"version\": \"https://jsonfeed.org/version/1\",
    \"title\": \"@{user}\",
    \"home_page_url\": \"{host}\",
    \"feed_url\": \"{host}/feed.json\",
    \"items\": [
        {feeds}
    ]
}}",
    user = USERNAME,
    host = HOSTNAME,
    feeds = {
        let mut ret = "".to_string();

        for post in posts {
            ret.push_str(&post.as_json());
        }

        ret
})
}

fn main() {
    println!(
        "# microbe for @{} (POSTS_FILE={}, HOSTNAME={})",
        USERNAME, POSTS_FILE, HOSTNAME
    );

    let posts = get_posts(POSTS_FILE);
    println!(
        "{}",
        create_rss_from_posts(&posts)
            .split('\n')
            .map(|x| format!("!feed.rss\t{}", x))
            .collect::<Vec<String>>()
            .join("\n")
    );
    println!(
        "{}",
        create_atom_from_posts(&posts)
            .split('\n')
            .map(|x| format!("!feed.atom\t{}", x))
            .collect::<Vec<String>>()
            .join("\n")
    );
    println!(
        "{}",
        create_json_from_posts(&posts)
            .split('\n')
            .map(|x| format!("!feed.json\t{}", x))
            .collect::<Vec<String>>()
            .join("\n")
    );
}

#[cfg(test)]
mod tests {
    use ::*;
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
            format!("<item>
    <title>1970-01-01T00:00:00+00:00</title>
    <description>Hello World, how are you today?</description>
    <link>{link}/0_hello-world-how-are-you</link>
    <guid isPermaLink=\"false\">86675203-8e40-5254-94ea-8c8f6f255bf1</guid>
</item>"
        , link = HOSTNAME));
    }

    #[test]
    fn post_to_atom() {
        assert_eq!(
            example_post().as_atom(),
            "<entry>
    <title>1970-01-01T00:00:00+00:00</title>
    <link type=\"text/html\" href=\"/0_hello-world-how-are-you\" />
    <id>urn:uuid:86675203-8e40-5254-94ea-8c8f6f255bf1</id>
    <updated>1970-01-01T00:00:00+00:00</updated>
    <author><name>microbe user</name></author>
    <content type=\"text\">
        Hello World, how are you today?
    </content>
</entry>"
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
user = USERNAME,
link = HOSTNAME,
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
user = USERNAME,
host = HOSTNAME,
now = Utc.timestamp(0, 0).to_rfc2822(),
id = uuid!(USERNAME).urn()))
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
user = USERNAME,
host = HOSTNAME));
}
}
