from pydantic_core import Email, Url, core_schema

from ..conftest import PyAndJson


def test_url_ok(py_and_json: PyAndJson):
    v = py_and_json(core_schema.url_schema())
    print(f'here si the schema: {v}')
    url = v.validate_test('https://example.com/foo/bar?baz=qux#quux')

    assert isinstance(url, Url)
    assert str(url) == 'https://example.com/foo/bar?baz=qux#quux'
    assert repr(url) == "Url('https://example.com/foo/bar?baz=qux#quux')"
    assert url.unicode_string() == 'https://example.com/foo/bar?baz=qux#quux'
    assert url.scheme == 'https'
    assert url.host == 'example.com'
    assert url.unicode_host() == 'example.com'
    assert url.path == '/foo/bar'
    assert url.query == 'baz=qux'
    assert url.query_params() == [('baz', 'qux')]
    assert url.fragment == 'quux'
    assert url.username is None
    assert url.password is None
    assert url.port == 443


def test_email_ok(py_and_json: PyAndJson):
    v = py_and_json(core_schema.email_schema())
    email = v.validate_test('levostatnigrosh@gmail.com')

    assert isinstance(email, Email)
    # assert email == 'levostatnigrosh@gmail.com'
